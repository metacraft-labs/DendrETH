use crate::serializers::{
    biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
    serde_bool_array_to_hex_string_nested,
};
use circuit_derive::{
    AddVirtualTarget, CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, SetWitness,
    TargetPrimitive,
};
use plonky2::{
    hash::hash_types::HashOutTarget, iop::target::BoolTarget,
    plonk::proof::ProofWithPublicInputsTarget,
};
use plonky2_crypto::biguint::BigUintTarget;

pub type Sha256Target = [BoolTarget; 256];
pub type SSZTarget = [BoolTarget; 256];
pub type Sha256MerkleBranchTarget<const DEPTH: usize> = [Sha256Target; DEPTH];
pub type PoseidonMerkleBranchTarget<const DEPTH: usize> = [HashOutTarget; DEPTH];
pub type PubkeyTarget = [BoolTarget; 384];
pub type SignatureTarget = [BoolTarget; 768];

#[derive(CircuitTarget, SerdeCircuitTarget)]
pub struct BasicRecursiveInnerCircuitTarget {
    pub proof1: ProofWithPublicInputsTarget<2>,
    pub proof2: ProofWithPublicInputsTarget<2>,
}

#[derive(
    Clone,
    Debug,
    TargetPrimitive,
    SetWitness,
    PublicInputsReadable,
    AddVirtualTarget,
    SerdeCircuitTarget,
)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorTarget {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,

    #[serde(with = "serde_bool_array_to_hex_string")]
    pub withdrawal_credentials: Sha256Target,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub effective_balance: BigUintTarget,

    pub slashed: BoolTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_eligibility_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub activation_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub exit_epoch: BigUintTarget,

    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub withdrawable_epoch: BigUintTarget,
}

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

#[derive(
    Clone,
    Debug,
    TargetPrimitive,
    SetWitness,
    PublicInputsReadable,
    AddVirtualTarget,
    SerdeCircuitTarget,
)]
#[serde(rename_all = "camelCase")]
pub struct DepositTargets {
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub deposit_index: BigUintTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub signature: SignatureTarget,
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub deposit_message_root: Sha256Target,
}
