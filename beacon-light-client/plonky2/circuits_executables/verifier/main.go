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

// type MyCircuit struct {
// 	A frontend.Variable
// 	B frontend.Variable
// 	C frontend.Variable `gnark:",public"`
// 	D frontend.Variable `gnark:",public"`
// }

// func (circuit *MyCircuit) Define(api frontend.API) error {
// 	// Define the circuit
// 	// a * b = c
// 	x := api.Mul(circuit.A, circuit.B)
// 	d := api.Add(circuit.A, circuit.B)

// 	api.AssertIsEqual(x, circuit.C)
// 	api.AssertIsEqual(d, circuit.D)
// 	return nil
// }

func main() {
	// var circuit MyCircuit
	// r1cs, _ := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)

	// log.Debug().Msg("Circuit compiled")

	// assignment := &MyCircuit{
	// 	A: 5,
	// 	B: 7,
	// 	C: 35,
	// 	D: 12,
	// }

	// witness, _ := frontend.NewWitness(assignment, ecc.BN254.ScalarField())

	// log.Debug().Msg("Circuit compiled")

	// srs, _ := test.NewKZGSRS(r1cs)
	// // filePath := "circuit/srs_setup"

	// // if _, err := os.Stat(filePath); os.IsNotExist(err) {
	// // 	trusted_setup.DownloadAndSaveAztecIgnitionSrs(174, filePath)
	// // }

	// // srs := kzg.NewSRS(ecc.BN254)

	// // fSrs, _ := os.Open(filePath)

	// // _, _ = srs.ReadFrom(fSrs)

	// log.Debug().Msg("test setup")

	// pk, vk, _ := plonk.Setup(r1cs, srs)

	// log.Debug().Msg("setup")

	// proof, _ := plonk.Prove(r1cs, pk, witness)

	// publicWitness, _ := witness.Public()

	// plonk.Verify(proof, vk, publicWitness)

	// fmt.Println("Public witness:", publicWitness)
	// w := publicWitness.Vector().(fr_bn254.Vector)
	// fmt.Println("My slice:", w)

	// contractFile, _ := os.Create("verifier.sol")
	// vk.ExportSolidity(contractFile)
	// contractFile.Close()

	// _proof := proof.(*plonk_bn254.Proof)
	// solidityBytes := _proof.MarshalSolidity()
	// proofFile, _ := os.Create("solidity_bytes.bin")
	// proofFile.Write(solidityBytes)
	// proofFile.Close()

	// proofStr := "0x" + hex.EncodeToString(_proof.MarshalSolidity())
	// fmt.Printf("proofStr: %v\n", proofStr)

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
