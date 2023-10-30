pub mod get_total_active_balance_first_level;

use std::println;

use casper_finality_proofs::{
    commitment_mapper_first_level::CommitmentMapperFirstLevel,
    validator::{ValidatorValue, ValidatorVariable},
};
use plonky2x::{
    backend::circuit::Circuit,
    frontend::hash::poseidon::poseidon256::PoseidonHashOutVariable,
    prelude::{Bytes32Variable, CircuitBuilder, DefaultParameters, GateRegistry, HintRegistry},
    utils::{self, bytes, bytes32},
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    CommitmentMapperFirstLevel::define(&mut builder);
    let circuit = builder.build();

    // let hint_serializer = HintRegistry::<DefaultParameters, 2>::new();
    // let gate_serializer = GateRegistry::<DefaultParameters, 2>::new();

    // circuit.save(
    //     &"build/first_level.circuit".to_string(),
    //     &gate_serializer,
    //     &hint_serializer,
    // );

    let mut input = circuit.input();
    input.write::<ValidatorVariable>( ValidatorValue{
        pubkey: bytes!("0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95"),
        withdrawal_credentials: bytes32!("0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50"),
        activation_epoch: 0,
        activation_eligibility_epoch: 0,
        exit_epoch: 18446744073709551615,
        slashed: false,
        effective_balance: 32000000000,
        withdrawable_epoch: 18446744073709551615,
    });
    input.write::<ValidatorVariable>( ValidatorValue{
        pubkey: bytes!("0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95"),
        withdrawal_credentials: bytes32!("0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50"),
        activation_epoch: 0,
        activation_eligibility_epoch: 0,
        exit_epoch: 18446744073709551615,
        slashed: false,
        effective_balance: 32000000000,
        withdrawable_epoch: 18446744073709551615,
    });
    input.write::<ValidatorVariable>( ValidatorValue{
        pubkey: bytes!("0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95"),
        withdrawal_credentials: bytes32!("0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50"),
        activation_epoch: 0,
        activation_eligibility_epoch: 0,
        exit_epoch: 18446744073709551615,
        slashed: false,
        effective_balance: 32000000000,
        withdrawable_epoch: 18446744073709551615,
    });
    input.write::<ValidatorVariable>( ValidatorValue{
        pubkey: bytes!("0x933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95"),
        withdrawal_credentials: bytes32!("0x0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50"),
        activation_epoch: 0,
        activation_eligibility_epoch: 0,
        exit_epoch: 18446744073709551615,
        slashed: false,
        effective_balance: 32000000000,
        withdrawable_epoch: 18446744073709551615,
    });

    utils::setup_logger();

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    // let sha256_result = output.read::<Bytes32Variable>();

    let poseidon_result = output.read::<PoseidonHashOutVariable>();

    // println!("sha256_result {:?}", sha256_result);
    println!("poseidon_result {:?}", poseidon_result);

    println!("proof public inputs {:?}", proof.public_inputs);
}
