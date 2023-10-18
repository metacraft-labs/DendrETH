use std::{fs, println};

use casper_finality_proofs::commitment_mapper_inner_level::define_commitment_mapper_inner_level;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2x::{
    backend::circuit::CircuitBuild,
    frontend::hash::poseidon::poseidon256::PoseidonHashOutVariable,
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters, GateRegistry, HintRegistry},
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

    define_commitment_mapper_inner_level(&mut commitment_mapper_inner_level_builder, &child_circuit);

    let proof1_bytes = fs::read("build/proof1").unwrap();
    let proof2_bytes = fs::read("build/proof2").unwrap();

    let proof1 =
        ProofWithPublicInputs::from_bytes(proof1_bytes, &child_circuit.data.common).unwrap();

    let proof2 =
        ProofWithPublicInputs::from_bytes(proof2_bytes, &child_circuit.data.common).unwrap();

    let circuit = commitment_mapper_inner_level_builder.build();

    let mut input = circuit.input();

    input.proof_write(proof1);
    input.proof_write(proof2);

    let (proof, mut output) = circuit.prove(&input);

    circuit.data.verify(proof).unwrap();

    println!("sha256_result {:?}", output.read::<Bytes32Variable>());
    println!(
        "poseidon result {:?}",
        output.read::<PoseidonHashOutVariable>()
    );
}
