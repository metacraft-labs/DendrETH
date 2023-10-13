use std::print;

use casper_finality_proofs::test::TestCircuit;
use plonky2x::{
    backend::{circuit::Circuit, function::VerifiableFunction},
    frontend::eth::beacon::vars::BeaconValidatorVariable,
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters, PlonkParameters, U64Variable},
    utils::{bytes32, eth::beacon::BeaconValidator},
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    TestCircuit::define(&mut builder);

    let circuit = builder.build();
    let mut input = circuit.input();

    input.write::<Bytes32Variable>(bytes32!(
        "0x1230000000000000000000000000000000000000000000000000000000000000"
    ));
    input.write::<Bytes32Variable>(bytes32!(
        "0x4560000000000000000000000000000000000000000000000000000000000000"
    ));

    input.write::<U64Variable>(6152);

    let value = BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: "0x1230000000000000000000000000000000000000000000000000000000000000".to_string(),
        activation_epoch: 6152,
        activation_eligibility_epoch: 6152,
        exit_epoch: "6152".to_string(),
        slashed: false,
        effective_balance: 32,
        withdrawable_epoch: "6152".to_string(),
    };

    input.write::<BeaconValidatorVariable>(value);

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let hash = output.read::<Bytes32Variable>();

    let epoch = output.read::<U64Variable>();

    let validator = output.read::<BeaconValidatorVariable>();

    println!("hash {:?}", hash);

    println!("epoch {:?}", epoch);

    println!("validator {:?}", validator);
}
