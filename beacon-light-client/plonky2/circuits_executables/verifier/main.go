package main

import (
	"encoding/hex"
	"flag"
	"fmt"
	"os"

	"github.com/consensys/gnark/backend/plonk"
	plonk_bn254 "github.com/consensys/gnark/backend/plonk/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/logger"
)

func main() {
	circuitPath := flag.String("circuit", "", "path to the circuit")
	dataPath := flag.String("data", "", "path to the data")
	saveProvingKey := flag.Bool("savepk", false, "save the proving key")
	loadProvingKey := flag.Bool("loadpk", false, "load the proving key")
	proofFlag := flag.Bool("proof", false, "create the proof")
	compileFlag := flag.Bool("compile", false, "compile the circuit")
	contractFlag := flag.Bool("contract", false, "Generate solidity contract")
	flag.Parse()

	log := logger.Logger()

	log.Debug().Msg("Circuit path: " + *circuitPath)
	log.Debug().Msg("Data path: " + *dataPath)
	log.Debug().Msg("Save proving key: " + fmt.Sprintf("%t", *saveProvingKey))
	log.Debug().Msg("Load proving key: " + fmt.Sprintf("%t", *loadProvingKey))
	log.Debug().Msg("Create proof: " + fmt.Sprintf("%t", *proofFlag))
	log.Debug().Msg("Compile circuit: " + fmt.Sprintf("%t", *compileFlag))
	log.Debug().Msg("Generate solidity contract: " + fmt.Sprintf("%t", *contractFlag))

	var r1cs constraint.ConstraintSystem
	var pk plonk.ProvingKey
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

	if *proofFlag {
		log.Info().Msg("loading the plonk proving key, circuit data and verifying key")

		if *loadProvingKey {
			r1cs, pk, vk, err = LoadCircuitData(*dataPath)

			if err != nil {
				log.Error().Msg("Failed to load circuit data: " + err.Error())
				os.Exit(1)
			}
		} else {
			r1cs, pk, vk, err = CompileVerifierCircuit(*circuitPath)

			if err != nil {
				log.Error().Msg("Failed to compile circuit: " + err.Error())
				os.Exit(1)
			}
		}

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
