use crate::hash_test::HashTestCircuit;
use crate::test_engine::types::test_hash_data::TestInput;
use crate::test_engine::utils::parse_file::read_fixture;
use crate::{assert_equal, to_string};
use once_cell::sync::Lazy;
use plonky2x::backend::circuit::CircuitBuild;
use plonky2x::frontend::eth::beacon::vars::BeaconValidatorVariable;
use plonky2x::prelude::{Bytes32Variable, U64Variable};
use plonky2x::utils::eth::beacon::BeaconValidator;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters},
};

// Singleton-like pattern
static CIRCUIT: Lazy<CircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    HashTestCircuit::define(&mut builder);
    builder.build()
});

pub fn wrapper(path: &str, should_assert: bool) -> Result<String, anyhow::Error> {
    let json_data: TestInput = read_fixture::<TestInput>(path);

    // a way to convert hex string to U256
    // let u256_from_str = U256::from_str_radix(json_data.inputs.str.as_str(), 16).unwrap();

    let mut input = CIRCUIT.input();

    input.write::<Bytes32Variable>(json_data.inputs.a);
    input.write::<Bytes32Variable>(json_data.inputs.b);

    let slot = json_data.inputs.slot.as_u64();
    input.write::<U64Variable>(slot);

    let value = BeaconValidator {
        pubkey: to_string!(json_data.inputs.pubkey),
        withdrawal_credentials: to_string!(json_data.inputs.a),
        activation_epoch: slot.to_string(),
        activation_eligibility_epoch: slot.to_string(),
        exit_epoch: slot.to_string(),
        slashed: json_data.inputs.slashed,
        effective_balance: 32,
        withdrawable_epoch: slot.to_string(),
    };

    input.write::<BeaconValidatorVariable>(value);

    let (proof, mut output) = CIRCUIT.prove(&input);

    CIRCUIT.verify(&proof, &input, &output);

    let hash = output.read::<Bytes32Variable>();

    let epoch = output.read::<U64Variable>();

    if should_assert {
        assert_equal!(hash, json_data.outputs.hash);
        assert_equal!(epoch, json_data.outputs.epoch.as_u64());
    }
    Ok(format!("{} {}", to_string!(hash), epoch))
}
