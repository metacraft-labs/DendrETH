use crate::{
    common_targets::Sha256Target,
    serializers::{biguint_to_str, parse_biguint, serde_bool_array_to_hex_string},
};
use circuit_derive::{PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
};
use plonky2_crypto::biguint::BigUintTarget;

use crate::common_targets::PubkeyTarget;

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorStatsTargets {
    pub non_activated_validators_count: Target,
    pub active_validators_count: Target,
    pub exited_validators_count: Target,
    pub slashed_validators_count: Target,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatedDataTargets {
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub balance_sum: BigUintTarget,
    pub deposits_count: Target,
    pub validator_stats: ValidatorStatsTargets,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct RangeObjectTarget {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub deposit_index: BigUintTarget,
    pub is_counted: BoolTarget,
    pub is_dummy: BoolTarget,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct NodeTargets {
    pub leftmost: RangeObjectTarget,
    pub rightmost: RangeObjectTarget,
    pub accumulated: AccumulatedDataTargets,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUintTarget,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub eth1_deposit_index: BigUintTarget,

    pub commitment_mapper_root: HashOutTarget,
    pub deposits_mapper_root: HashOutTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balances_root: Sha256Target,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub genesis_fork_version: [BoolTarget; 32],
}
