use crate::{
    common_targets::ValidatorTarget,
    serializers::{serde_bool_array_to_hex_string, serde_bool_array_to_hex_string_nested},
    utils::circuit::hashing::merkle::{
        poseidon::hash_validator_poseidon_or_zeroes,
        sha256::{hash_validator_sha256_or_zeroes, merklelize_validator_target},
    },
};
use circuit::Circuit;
use circuit_derive::{CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};

use crate::common_targets::{SSZTarget, Sha256Target};

#[derive(TargetPrimitive, PublicInputsReadable)]
pub struct MerklelizedValidatorTarget {
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub pubkey: [SSZTarget; 2],
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub effective_balance: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub slashed: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub activation_eligibility_epoch: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub activation_epoch: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub exit_epoch: SSZTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawable_epoch: SSZTarget,
}

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperTarget {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256_hash_tree_root: Sha256Target,

    #[target(out)]
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub struct ValidatorsCommitmentMapperFirstLevel {}

impl Circuit for ValidatorsCommitmentMapperFirstLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = ValidatorsCommitmentMapperTarget;

    type Params = ();

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        _params: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        let merklelized_validator = merklelize_validator_target(builder, &input.validator);
        let sha256_hash_tree_root =
            hash_validator_sha256_or_zeroes(builder, &merklelized_validator, input.is_real);

        let poseidon_hash_tree_root =
            hash_validator_poseidon_or_zeroes(builder, &input.validator, input.is_real);

        Self::Target {
            validator: input.validator,
            is_real: input.is_real,
            sha256_hash_tree_root,
            poseidon_hash_tree_root,
        }
    }
}
