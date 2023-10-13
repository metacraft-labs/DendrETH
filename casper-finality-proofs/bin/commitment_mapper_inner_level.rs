use casper_finality_proofs::{
    proof_utils::ProofWithPublicInputsTargetReader,
};

use plonky2x::{
    backend::{
        circuit::{CircuitBuild},
    },
    frontend::{
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{
        Bytes32Variable, CircuitBuilder, DefaultParameters, GateRegistry, HintRegistry,
    },
};

fn main() {
    let mut commitment_mapper_inner_level_builder = CircuitBuilder::<DefaultParameters, 2>::new();

    let generator_registry = HintRegistry::<DefaultParameters, 2>::new();
    let gate_registry = GateRegistry::<DefaultParameters, 2>::new();

    let child_circuit = CircuitBuild::<DefaultParameters, 2>::load(
        "build/first_level.circuit",
        &gate_registry,
        &generator_registry,
    )
    .unwrap();

    let verifier_data = commitment_mapper_inner_level_builder
        .constant_verifier_data::<DefaultParameters>(&child_circuit.data);
    let proof1 = commitment_mapper_inner_level_builder
        .proof_read(&child_circuit)
        .into();
    let proof2 = commitment_mapper_inner_level_builder
        .proof_read(&child_circuit)
        .into();

    commitment_mapper_inner_level_builder.verify_proof::<DefaultParameters>(
        &proof1,
        &verifier_data,
        &child_circuit.data.common,
    );

    commitment_mapper_inner_level_builder.verify_proof::<DefaultParameters>(
        &proof2,
        &verifier_data,
        &child_circuit.data.common,
    );

    let mut proof1_reader = ProofWithPublicInputsTargetReader::from(proof1);
    let mut proof2_reader = ProofWithPublicInputsTargetReader::from(proof2);

    let sha256_hash1 = proof1_reader.read::<Bytes32Variable>();
    let poseidon_hash = proof1_reader.read::<PoseidonHashOutVariable>();

    let sha256_hash2 = proof2_reader.read::<Bytes32Variable>();
    let poseidon_hash2 = proof2_reader.read::<PoseidonHashOutVariable>();

    let sha256 = commitment_mapper_inner_level_builder.sha256_pair(sha256_hash1, sha256_hash2);
    let poseidon =
        commitment_mapper_inner_level_builder.poseidon_hash_pair(poseidon_hash, poseidon_hash2);

    commitment_mapper_inner_level_builder.write(sha256);
    commitment_mapper_inner_level_builder.write(poseidon);
}
