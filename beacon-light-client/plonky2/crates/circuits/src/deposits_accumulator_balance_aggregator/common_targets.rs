use crate::serializers::{biguint_to_str, parse_biguint, serde_bool_array_to_hex_string};
use circuit_derive::{PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use plonky2::iop::target::{BoolTarget, Target};
use plonky2_crypto::biguint::BigUintTarget;

use crate::common_targets::PubkeyTarget;

#[derive(Clone, Copy, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorStatusStatsTarget {
    pub non_activated_count: Target,
    pub active_count: Target,
    pub exited_count: Target,
    pub slashed_count: Target,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatedDataTarget {
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub balance: BigUintTarget,
    pub deposits_count: Target,
    pub validator_status_stats: ValidatorStatusStatsTarget,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatedDataTargetDiva {
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub balance: BigUintTarget,
    pub validator_status_stats: ValidatorStatusStatsTarget,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDataTarget {
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub balance: BigUintTarget,

    pub is_non_activated: BoolTarget,
    pub is_active: BoolTarget,
    pub is_exited: BoolTarget,
    pub is_slashed: BoolTarget,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositDataTarget {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,
    pub validator: ValidatorDataTarget,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub deposit_index: BigUintTarget,
    pub is_counted: BoolTarget,
    pub is_dummy: BoolTarget,
}
