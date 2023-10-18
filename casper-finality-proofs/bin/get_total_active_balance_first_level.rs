use std::println;

use casper_finality_proofs::get_total_active_balance_first_level::TotalActiveBalanceFirstLevel;
use ethers::types::U256;
use plonky2x::{
    backend::circuit::Circuit,
    frontend::{
        eth::beacon::vars::BeaconValidatorVariable,
        hash::poseidon::poseidon256::PoseidonHashOutVariable,
    },
    prelude::{
        ArrayVariable, Bytes32Variable, CircuitBuilder, DefaultParameters, GateRegistry,
        HintRegistry, U256Variable, U64Variable,
    },
    utils::{bytes32, eth::beacon::BeaconValidator, self},
};

fn main() {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    TotalActiveBalanceFirstLevel::define(&mut builder);
    let circuit = builder.build();

    let hint_serializer = HintRegistry::<DefaultParameters, 2>::new();
    let gate_serializer = GateRegistry::<DefaultParameters, 2>::new();

    circuit.save(
        &"build/total_active_balance_first_level.circuit".to_string(),
        &gate_serializer,
        &hint_serializer,
    );

    let mut input = circuit.input();

    input.write::<ArrayVariable<BeaconValidatorVariable, 8>>( vec![BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: "0x1230000000000000000000000000000000000000000000000000000000000000".to_string(),
        activation_epoch: "6152".to_string(),
        activation_eligibility_epoch: "6152".to_string(),
        exit_epoch: "6153".to_string(),
        slashed: false,
        effective_balance: 32,
        withdrawable_epoch: "6152".to_string(),
    }; 8]);

    input.write::<ArrayVariable<Bytes32Variable, 2>>(vec![
        bytes32!("0x1230000000000000000000000000000000000000000000000000000000000000"),
        bytes32!("0x1230000000000000000000000000000000000000000000000000000000000000"),
    ]);

    input.write::<U256Variable>(U256::from(6152));

    utils::setup_logger();

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let sum = output.read::<U64Variable>();

    let current_epoch = output.read::<U256Variable>();

    let validators_poseidon_root = output.read::<PoseidonHashOutVariable>();

    let balances_root = output.read::<Bytes32Variable>();

    println!("sum: {:?}", sum);

    println!("current_epoch: {:?}", current_epoch);

    println!("validators_poseidon_root: {:?}", validators_poseidon_root);

    println!("balances_root: {:?}", balances_root);
}
