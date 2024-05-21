use crate::serializers::serde_bool_array_to_hex_string;
use crate::utils::hashing::sha256::sha256;
use crate::validators_commitment_mapper::first_level::{
    merklelize_validator_target, MerklelizedValidatorTarget,
};
use circuit::Circuit;
use circuit_derive::CircuitTarget;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;

use crate::common_targets::Sha256Target;
use crate::utils::hashing::validator_hash_tree_root::{
    hash_validator_sha256, hash_validator_sha256_or_zeroes,
};
use crate::utils::hashing::validator_hash_tree_root_poseidon::ValidatorTarget;

#[derive(CircuitTarget)]
pub struct TestTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(out)]
    pub merklelized_validator: MerklelizedValidatorTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials_sha256: Sha256Target,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub hash_tree_root: Sha256Target,
}

pub struct TestCircuit {}

const D: usize = 2;

impl Circuit for TestCircuit {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = TestTarget;
    type Params = ();

    fn define(builder: &mut CircuitBuilder<Self::F, D>, _params: &()) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let merklelized_validator = merklelize_validator_target(builder, &input.validator);
        let hash_tree_root = hash_validator_sha256(builder, &merklelized_validator);

        let withdrawal_credentials_sha256 =
            sha256(builder, &input.validator.withdrawal_credentials);

        Self::Target {
            validator: input.validator,
            merklelized_validator,
            hash_tree_root,
            withdrawal_credentials_sha256,
        }
    }
}

#[cfg(test)]
mod test {
    use circuit::{set_witness::SetWitness, Circuit};
    use plonky2::iop::witness::PartialWitness;

    use super::TestCircuit;

    #[test]
    fn some_test() {
        let input = serde_json::from_str(
            r#"
{"validator":{"pubkey":"933ad9491b62059dd065b560d256d8957a8c402cc6e8d8ee7290ae11e8f7329267a8811c397529dac52ae1342ba58c95","withdrawalCredentials":"0100000000000000000000000d369bb49efa5100fd3b86a9f828c55da04d2d50","effectiveBalance":"32000000000","slashed":false,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"}}
        "#,
        ).unwrap();

        let (target, data) = TestCircuit::build(&());

        let mut pw = PartialWitness::new();
        target.set_witness(&mut pw, &input);
        let proof = data.prove(pw).unwrap();
        let public_inputs = TestCircuit::read_public_inputs(&proof.public_inputs);
        println!("result: {:?}", serde_json::to_string(&public_inputs));
    }
}
