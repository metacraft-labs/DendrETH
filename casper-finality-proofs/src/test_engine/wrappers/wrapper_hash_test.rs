use crate::hash_test::HashTestCircuit;
use crate::test_engine::types::test_hash_data::TestInput;
use crate::test_engine::utils::parse_file::read_fixture;
use plonky2x::frontend::eth::beacon::vars::BeaconValidatorVariable;
use plonky2x::prelude::{Bytes32Variable, U64Variable};
use plonky2x::utils::eth::beacon::BeaconValidator;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters},
};

pub fn wrapper(path: &str) {
    let json_data: TestInput = read_fixture::<TestInput>(path);

    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    HashTestCircuit::define(&mut builder);

    let circuit = builder.build();
    let mut input = circuit.input();

    input.write::<Bytes32Variable>(json_data.inputs.a);
    input.write::<Bytes32Variable>(json_data.inputs.b);

    let slot = json_data.inputs.slot.as_u64();
    input.write::<U64Variable>(slot);

    let a_str = json_data
        .inputs
        .a
        .as_bytes()
        .iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();

    let value = BeaconValidator {
        pubkey: "0x123000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string(),
        withdrawal_credentials: a_str,
        activation_epoch: slot,
        activation_eligibility_epoch: slot,
        exit_epoch: slot.to_string(),
        slashed: false,
        effective_balance: 32,
        withdrawable_epoch: slot.to_string(),
    };

    input.write::<BeaconValidatorVariable>(value);

    let (proof, mut output) = circuit.prove(&input);

    circuit.verify(&proof, &input, &output);

    let hash = output.read::<Bytes32Variable>();

    let epoch = output.read::<U64Variable>();

    assert_eq!(hash, json_data.outputs.hash);
    assert_eq!(epoch, json_data.outputs.epoch.as_u64());
}
