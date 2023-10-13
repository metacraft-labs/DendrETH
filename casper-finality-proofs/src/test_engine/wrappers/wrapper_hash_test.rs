use crate::hash_test::HashTestCircuit;
use crate::test_engine::types::test_hash_data::TestInput;
use crate::test_engine::utils::parse_file::read_fixture;
use crate::{assert_equal, to_string};
use plonky2x::frontend::eth::beacon::vars::BeaconValidatorVariable;
use plonky2x::prelude::{Bytes32Variable, U64Variable};
use plonky2x::utils::eth::beacon::BeaconValidator;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters},
};

pub fn wrapper(path: &str) -> Result<(), anyhow::Error> {
    let json_data: TestInput = read_fixture::<TestInput>(path);

    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    HashTestCircuit::define(&mut builder);

    let circuit = builder.build();
    let mut input = circuit.input();

    input.write::<Bytes32Variable>(json_data.inputs.a);
    input.write::<Bytes32Variable>(json_data.inputs.b);

    let slot = json_data.inputs.slot.as_u64();
    input.write::<U64Variable>(slot);

    let value = BeaconValidator {
        pubkey: to_string!(json_data.inputs.pubkey),
        withdrawal_credentials: to_string!(json_data.inputs.a),
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

    assert_equal!(hash, json_data.outputs.hash);
    assert_equal!(epoch, json_data.outputs.epoch.as_u64());
    Ok(())
}
