pub mod get_total_active_balance_first_level;

use std::println;

use casper_finality_proofs::commitment_mapper_first_level::CommitmentMapperFirstLevel;
use plonky2x::{
    backend::circuit::Circuit,
    frontend::{
        eth::beacon::vars::BeaconValidatorVariable,
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters, GateRegistry, HintRegistry},
    utils::eth::beacon::BeaconValidator,
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    CommitmentMapperFirstLevel::define(&mut builder);
    let circuit = builder.build();

    let hint_serializer = HintRegistry::<DefaultParameters, 2>::new();
    let gate_serializer = GateRegistry::<DefaultParameters, 2>::new();

    circuit.save(
        &"build/first_level.circuit".to_string(),
        &gate_serializer,
        &hint_serializer,
    );

    let mut input = circuit.input();
    input.write::<BeaconValidatorVariable>( BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: "0x1230000000000000000000000000000000000000000000000000000000000000".to_string(),
        activation_epoch: "6152".to_string(),
        activation_eligibility_epoch: "6152".to_string(),
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
}
