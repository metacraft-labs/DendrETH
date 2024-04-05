package main

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"net/http"
	"os"
	"sync"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/plonk"
	plonk_bn254 "github.com/consensys/gnark/backend/plonk/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/logger"
	"github.com/succinctlabs/gnark-plonky2-verifier/types"
	"github.com/succinctlabs/gnark-plonky2-verifier/variables"
)

const PORT = 3333

var (
	r1cs constraint.ConstraintSystem
	pk   plonk.ProvingKey
)

func main() {
	circuitPath := flag.String("circuit", "", "path to the circuit")
	dataPath := flag.String("data", "", "path to the data")
	saveProvingKey := flag.Bool("savepk", false, "save the proving key")
	loadProvingKey := flag.Bool("loadpk", false, "load the proving key")
	proofFlag := flag.Bool("proof", false, "create the proof")
	compileFlag := flag.Bool("compile", false, "compile the circuit")
	contractFlag := flag.Bool("contract", false, "Generate solidity contract")
	startServerFlag := flag.Bool("server", false, "Start an http proving server")
	flag.Parse()

	log := logger.Logger()
	log.Debug().Msg("Circuit path: " + *circuitPath)
	log.Debug().Msg("Data path: " + *dataPath)
	log.Debug().Msg("Save proving key: " + fmt.Sprintf("%t", *saveProvingKey))
	log.Debug().Msg("Load proving key: " + fmt.Sprintf("%t", *loadProvingKey))
	log.Debug().Msg("Create proof: " + fmt.Sprintf("%t", *proofFlag))
	log.Debug().Msg("Compile circuit: " + fmt.Sprintf("%t", *compileFlag))
	log.Debug().Msg("Generate solidity contract: " + fmt.Sprintf("%t", *contractFlag))
	log.Debug().Msg("Start an http proving server: " + fmt.Sprintf("%t", *startServerFlag))

	defer func() {
		if !*startServerFlag {
			return
		}

		http.Handle("/genProof", newGenerateProofHandler())
		log.Info().Msg(fmt.Sprintf("Listening on port: %v", PORT))
		if err := http.ListenAndServe(fmt.Sprintf(":%v", PORT), nil); err != nil {
			if errors.Is(err, http.ErrServerClosed) {
				log.Info().Msg("Server closed")
			} else if err != nil {
				log.Error().Msg(fmt.Sprintf("Error starting server: %s", err.Error()))
				os.Exit(1)
			}
		}
	}()

	var vk plonk.VerifyingKey
	var err error

	if *compileFlag {
		log.Info().Msg("Compiling circuit")

		r1cs, pk, vk, err = CompileVerifierCircuit(*circuitPath)

		if err != nil {
			log.Error().Msg("Failed to compile circuit: " + err.Error())
			os.Exit(1)
		}

		if *saveProvingKey {
			SaveVerifierCircuit(*dataPath, r1cs, pk, vk)
		}
	}

	if *loadProvingKey {
		log.Info().Msg("loading the plonk proving key, circuit data and verifying key")

		r1cs, pk, vk, err = LoadCircuitData(*dataPath)

		if err != nil {
			log.Error().Msg("Failed to load circuit data: " + err.Error())
			os.Exit(1)
		}
	}

	if *proofFlag {
		log.Info().Msg("Generating proof")
		proof, publicWitness, err := Prove(*circuitPath, r1cs, pk)

		_proof := proof.(*plonk_bn254.Proof)
		proofStr := "0x" + hex.EncodeToString(_proof.MarshalSolidity())
		fmt.Printf("proofStr: %v\n", proofStr)

		if err != nil {
			log.Error().Msg("Failed to generate proof: " + err.Error())
			os.Exit(1)
		}

		log.Info().Msg("Verifying proof")

		err = plonk.Verify(proof, vk, publicWitness)
		if err != nil {
			log.Error().Msg("Failed to verify proof: " + err.Error())
			os.Exit(1)
		}

		log.Info().Msg("Successfully verified proof")
	}

	if *contractFlag {
		log.Info().Msg("Generating solidity contract")
		ExportSolidityContract(*dataPath, vk)
	}
}

type GenerateProofDTO struct {
	VerifierOnlyCircuitData types.VerifierOnlyCircuitDataRaw `json:"verifier_only_circuit_data"`
	ProofWithPublicInputs   types.ProofWithPublicInputsRaw   `json:"proof_with_public_inputs"`
}

type GenerateProofHandler struct {
	waitingForJobCV *sync.Cond
	waitingForJob   *bool
}

func newGenerateProofHandler() GenerateProofHandler {
	return GenerateProofHandler{
		waitingForJobCV: sync.NewCond(&sync.Mutex{}),
		waitingForJob: func() *bool {
			flag := true
			return &flag
		}(),
	}
}

func (handler GenerateProofHandler) ServeHTTP(res http.ResponseWriter, req *http.Request) {
	log := logger.Logger()

	handler.waitingForJobCV.L.Lock()
	for !*handler.waitingForJob {
		handler.waitingForJobCV.Wait()
	}

	*handler.waitingForJob = false

	defer func() {
		*handler.waitingForJob = true
		handler.waitingForJobCV.L.Unlock()
		handler.waitingForJobCV.Signal()
	}()

	var dto GenerateProofDTO
	if err := json.NewDecoder(req.Body).Decode(&dto); err != nil {
		http.Error(res, fmt.Sprintf("Error while parsing request body: %s", err.Error()), http.StatusBadRequest)
		return
	}

	verifierOnlyCircuitData := variables.DeserializeVerifierOnlyCircuitData(dto.VerifierOnlyCircuitData)
	proofWithPisVariable := variables.DeserializeProofWithPublicInputs(dto.ProofWithPublicInputs)
	publicInputHash := GetPublicInputHash(dto.ProofWithPublicInputs.PublicInputs)

	assignment := Plonky2VerifierCircuit{
		ProofWithPis:    proofWithPisVariable,
		VerifierData:    verifierOnlyCircuitData,
		VerifierDigest:  verifierOnlyCircuitData.CircuitDigest,
		PublicInputHash: publicInputHash,
	}

	log.Debug().Msg("Generating witness")
	witness, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		http.Error(res, fmt.Sprintf("failed to generate witness: %s", err.Error()), http.StatusBadRequest)
		return
	}

	log.Debug().Msg("Successfully generated witness")
	log.Debug().Msg("Creating proof")

	proof, err := plonk.Prove(r1cs, pk, witness)
	if err != nil {
		http.Error(res, fmt.Sprintf("failed to create proof: %s", err.Error()), http.StatusBadRequest)
		return
	}

	log.Debug().Msg("Successfully created proof")

	res.Header().Add("Content-Type", "octet-stream")
	res.Write(proof.(*plonk_bn254.Proof).MarshalSolidity())
}
