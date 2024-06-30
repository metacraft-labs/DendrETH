use crate::serializers::serde_bool_array_to_hex_string;
use circuit::{serde::serde_u64_str, targets::uint::Uint64Target};
use circuit_derive::{PublicInputsReadable, SerdeCircuitTarget, TargetPrimitive};
use plonky2::iop::target::{BoolTarget, Target};

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
    #[serde(with = "serde_u64_str")]
    pub balance: Uint64Target,
    pub deposits_count: Target,
    pub validator_status_stats: ValidatorStatusStatsTarget,
}

#[derive(Clone, Debug, TargetPrimitive, PublicInputsReadable, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorDataTarget {
    #[serde(with = "serde_u64_str")]
    pub balance: Uint64Target,

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
    #[serde(with = "serde_u64_str")]
    pub deposit_index: Uint64Target,
    pub is_counted: BoolTarget,
    pub is_dummy: BoolTarget,
}
