package main

import (
	"fmt"
	"math/big"
	"os"
	"time"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/kzg"
	"github.com/consensys/gnark/backend/plonk"
	plonk_bn254 "github.com/consensys/gnark/backend/plonk/bn254"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/logger"
	"github.com/rs/zerolog/log"
	"github.com/succinctlabs/gnark-plonky2-verifier/trusted_setup"
	"github.com/succinctlabs/gnark-plonky2-verifier/types"
	"github.com/succinctlabs/gnark-plonky2-verifier/variables"
	"github.com/succinctlabs/gnark-plonky2-verifier/verifier"
)

type Plonky2VerifierCircuit struct {
	// Public inputs to the circuit
	VerifierDigest frontend.Variable `gnark:"verifierDigest,public"`

	PublicInputHash frontend.Variable `gnark:"publicInputHash,public"`

	// Private inputs to the circuit
	ProofWithPis variables.ProofWithPublicInputs
	VerifierData variables.VerifierOnlyCircuitData

	// Circuit configuration
	CommonCircuitData types.CommonCircuitData `gnark:"-"`
}

func (c *Plonky2VerifierCircuit) Define(api frontend.API) error {
	// initialize the verifier chip
	verifierChip := verifier.NewVerifierChip(api, c.CommonCircuitData)
	// verify the plonky2 proof
	verifierChip.Verify(c.ProofWithPis.Proof, c.ProofWithPis.PublicInputs, c.VerifierData)

	// Public inputs should be 32 bytes
	// big-endian representation of SHA256 hash that has been truncated to 253 bits
	publicInputs := c.ProofWithPis.PublicInputs

	if len(publicInputs) != 32 {
		return fmt.Errorf("expected 32 public inputs, got %d", len(publicInputs))
	}

	inputDigest := frontend.Variable(0)

	for i := 0; i < 32; i++ {
		pubByte := publicInputs[31-i].Limb
		inputDigest = api.Add(inputDigest, api.Mul(pubByte, frontend.Variable(new(big.Int).Lsh(big.NewInt(1), uint(8*i)))))
	}

	api.AssertIsEqual(c.PublicInputHash, inputDigest)

	// We have to assert that the VerifierData we verified the proof with
	// matches the VerifierDigest public input
	api.AssertIsEqual(c.VerifierDigest, c.VerifierData.CircuitDigest)

	return nil
}

func CompileVerifierCircuit(circuitPath string) (constraint.ConstraintSystem, plonk.ProvingKey, plonk.VerifyingKey, error) {
	log := logger.Logger()

	verifierOnlyCircuitData := variables.DeserializeVerifierOnlyCircuitData(types.ReadVerifierOnlyCircuitData(circuitPath + "/verifier_only_circuit_data.json"))

	proofWithPis := variables.DeserializeProofWithPublicInputs(types.ReadProofWithPublicInputs(circuitPath + "/proof_with_public_inputs.json"))

	commonCircuitData := types.ReadCommonCircuitData(circuitPath + "/common_circuit_data.json")

	circuit := Plonky2VerifierCircuit{
		ProofWithPis:      proofWithPis,
		VerifierData:      verifierOnlyCircuitData,
		PublicInputHash:   new(frontend.Variable),
		VerifierDigest:    new(frontend.Variable),
		CommonCircuitData: commonCircuitData,
	}

	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), scs.NewBuilder, &circuit)

	if err != nil {
		return nil, nil, nil, err
	}

	log.Info().Msg("Loading SRS setup")
	start := time.Now()

	filePath := circuitPath + "/" + "srs_setup"

	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		trusted_setup.DownloadAndSaveAztecIgnitionSrs(174, filePath)
	}

	srs := kzg.NewSRS(ecc.BN254)

	fSrs, _ := os.Open(filePath)

	_, err = srs.ReadFrom(fSrs)

	fSrs.Close()

	if err != nil {
		return nil, nil, nil, err
	}

	elapsed := time.Since(start)
	log.Info().Msg("Successfully loaded SRS setup time: " + elapsed.String())

	log.Info().Msg("Running circuit setup")

	start = time.Now()

	pk, vk, err := plonk.Setup(r1cs, srs)

	if err != nil {
		return nil, nil, nil, err
	}

	elapsed = time.Since(start)

	log.Info().Msg("Successfully ran circuit setup, time: " + elapsed.String())

	return r1cs, pk, vk, nil
}

func SaveVerifierCircuit(path string, r1cs constraint.ConstraintSystem, pk plonk.ProvingKey, vk plonk.VerifyingKey) {
	log := logger.Logger()

	err := os.MkdirAll(path, 0777)

	if err != nil {
		log.Error().Msg("Failed to create directory: " + err.Error())
		os.Exit(1)
	}

	log.Info().Msg("Saving r1cs to " + path)

	r1csFile, err := os.Create(path + "/r1cs.bin")
	r1cs.WriteTo(r1csFile)
	r1csFile.Close()

	if err != nil {
		log.Error().Msg("Failed to save r1cs: " + err.Error())
		os.Exit(1)
	}

	log.Info().Msg("Successfully saved r1cs")

	log.Info().Msg("Saving pk and vk to " + path)

	pkFile, err := os.Create(path + "/pk.bin")
	pk.WriteRawTo(pkFile)
	pkFile.Close()

	if err != nil {
		log.Error().Msg("Failed to save pk: " + err.Error())
		os.Exit(1)
	}

	vkFile, err := os.Create(path + "/vk.bin")
	vk.WriteRawTo(vkFile)
	vkFile.Close()

	if err != nil {
		log.Error().Msg("Failed to save vk: " + err.Error())
		os.Exit(1)
	}

	log.Info().Msg("Successfully saved pk and vk")
}

func LoadCircuitData(path string) (constraint.ConstraintSystem, plonk.ProvingKey, plonk.VerifyingKey, error) {
	r1csFile, err := os.Open(path + "/r1cs.bin")
	if err != nil {
		return nil, nil, nil, err
	}
	r1cs := plonk.NewCS(ecc.BN254)
	start := time.Now()
	_, err = r1cs.ReadFrom(r1csFile)
	if err != nil {
		return nil, nil, nil, err
	}
	elapsed := time.Since(start)
	r1csFile.Close()
	log.Debug().Msg("Successfully loaded r1cs, time: " + elapsed.String())

	pkFile, err := os.Open(path + "/pk.bin")
	if err != nil {
		return nil, nil, nil, err
	}
	pk := plonk.NewProvingKey(ecc.BN254)
	start = time.Now()
	if err != nil {
		return nil, nil, nil, err
	}
	_, err = pk.ReadFrom(pkFile)
	if err != nil {
		return nil, nil, nil, err
	}
	pkFile.Close()
	elapsed = time.Since(start)
	log.Debug().Msg("Successfully loaded pk, time: " + elapsed.String())

	vkFile, err := os.Open(path + "/vk.bin")
	if err != nil {
		return nil, nil, nil, err
	}
	vk := plonk.NewVerifyingKey(ecc.BN254)
	start = time.Now()
	_, err = vk.ReadFrom(vkFile)
	if err != nil {
		return nil, nil, nil, err
	}
	vkFile.Close()
	elapsed = time.Since(start)
	log.Debug().Msg("Successfully loaded vk, time: " + elapsed.String())

	return r1cs, pk, vk, nil
}

func GetPublicInputHash(publicInputs []uint64) frontend.Variable {
	if len(publicInputs) != 32 {
		panic("publicInputs must be 32 bytes")
	}
	publicInputsBytes := make([]byte, 32)
	for i, v := range publicInputs {
		publicInputsBytes[i] = byte(v & 0xFF)
	}
	publicInputHash := new(big.Int).SetBytes(publicInputsBytes[0:32])
	log.Debug().Msg("Public input hash len: " + fmt.Sprintf("%d", publicInputHash.BitLen()))
	if publicInputHash.BitLen() > 253 {
		panic("inputHash must be at most 253 bits")
	}
	return publicInputHash
}

func Prove(circuitPath string, r1cs constraint.ConstraintSystem, pk plonk.ProvingKey) (plonk.Proof, witness.Witness, error) {
	verifierOnlyCircuitData := variables.DeserializeVerifierOnlyCircuitData(types.ReadVerifierOnlyCircuitData(circuitPath + "/verifier_only_circuit_data.json"))

	proofWithPisRaw := types.ReadProofWithPublicInputs(circuitPath + "/proof_with_public_inputs.json")
	proofWithPisVariable := variables.DeserializeProofWithPublicInputs(proofWithPisRaw)

	publicInputHash := GetPublicInputHash(proofWithPisRaw.PublicInputs)

	assignment := Plonky2VerifierCircuit{
		ProofWithPis:    proofWithPisVariable,
		VerifierData:    verifierOnlyCircuitData,
		VerifierDigest:  verifierOnlyCircuitData.CircuitDigest,
		PublicInputHash: publicInputHash,
	}

	log.Debug().Msg("Generating witness")

	start := time.Now()

	witness, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate witness: %w", err)
	}
	elapsed := time.Since(start)
	log.Debug().Msg("Successfully generated witness, time: " + elapsed.String())

	log.Debug().Msg("Creating proof")
	start = time.Now()
	proof, err := plonk.Prove(r1cs, pk, witness)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create proof: %w", err)
	}
	elapsed = time.Since(start)
	log.Info().Msg("Successfully created proof, time: " + elapsed.String())

	log.Info().Msg("Saving proof to proof.json")

	_proof := proof.(*plonk_bn254.Proof)
	solidityBytes := _proof.MarshalSolidity()
	proofFile, _ := os.Create(circuitPath + "/solidity_bytes.bin")
	proofFile.Write(solidityBytes)
	proofFile.Close()

	log.Info().Msg("Successfully saved proof")

	publicWitness, err := witness.Public()
	log.Info().Msg("Saving public witness to public_witness.bin")
	witnessFile, err := os.Create("public_witness.bin")
	publicWitness.WriteTo(witnessFile)
	witnessFile.Close()
	log.Info().Msg("Successfully saved public witness")

	return proof, publicWitness, nil
}

func ExportSolidityContract(path string, vk plonk.VerifyingKey) {
	contractFile, _ := os.Create(path + "/verifier.sol")
	vk.ExportSolidity(contractFile)
	contractFile.Close()
}
