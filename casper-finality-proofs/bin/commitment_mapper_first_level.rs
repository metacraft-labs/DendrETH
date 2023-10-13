use std::println;

use casper_finality_proofs::{
    commitment_mapper_first_level::CommitmentMapperFirstLevel,
    commitment_mapper_inner_level::define_inner_level,
};
use plonky2::gates::poseidon;
use plonky2x::{
    backend::circuit::Circuit,
    frontend::{
        eth::beacon::vars::BeaconValidatorVariable,
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters},
    utils::eth::beacon::BeaconValidator,
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    CommitmentMapperFirstLevel::define(&mut builder);
    let circuit = builder.build();

    let mut input = circuit.input();
    input.write::<BeaconValidatorVariable>( BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: "0x1230000000000000000000000000000000000000000000000000000000000000".to_string(),
        activation_epoch: 6152,
        activation_eligibility_epoch: 6152,
        exit_epoch: "6152".to_string(),
        slashed: false,
        effective_balance: 32,
        withdrawable_epoch: "6152".to_string(),
    });

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let sha256_result = output.read::<Bytes32Variable>();

    let poseidon_result = output.read::<PoseidonHashOutVariable>();

    println!("sha256_result {:?}", sha256_result);
    println!("poseidon_result {:?}", poseidon_result);

    println!("proof public inputs {:?}", proof.public_inputs);

    let mut inner_level_builder = CircuitBuilder::<DefaultParameters, 2>::new();

    define_inner_level(&mut inner_level_builder, circuit);

    let inner_circuit = inner_level_builder.build();
    let mut inner_input = inner_circuit.input();

    inner_input.proof_write(proof.clone());
    inner_input.proof_write(proof);

    let (inner_proof, mut inner_output) = inner_circuit.prove(&inner_input);

    inner_circuit.verify(&inner_proof, &inner_input, &inner_output);

    // println!("sha256_result {:?}", inner_output.read::<Bytes32Variable>());
    // println!(
    //     "poseidon_result {:?}",
    //     inner_output.read::<PoseidonHashOutVariable>()
    // );
}
