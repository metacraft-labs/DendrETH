use crate::serializers::serde_bool_array_to_hex_string;
use crate::serializers::{biguint_to_str, parse_biguint};
use circuit_derive::{
    AddVirtualTarget, CircuitTarget, PublicInputsReadable, SerdeCircuitTarget, SetWitness,
    TargetPrimitive,
};
use plonky2::{iop::target::BoolTarget, plonk::proof::ProofWithPublicInputsTarget};
use plonky2_crypto::biguint::BigUintTarget;

pub type Sha256Target = [BoolTarget; 256];
pub type SSZLeafTarget = [BoolTarget; 256];
pub type Sha256MerkleBranchTarget<const DEPTH: usize> = [Sha256Target; DEPTH];

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
    pub pubkey: [BoolTarget; 384],

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
