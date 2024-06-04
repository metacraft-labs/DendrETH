use crate::{
    common_targets::Sha256Target,
    serializers::serde_bool_array_to_hex_string,
    utils::circuit::hashing::{poseidon::poseidon_or_zeroes, sha256::sha256_or_zeroes},
};
use circuit::{Circuit, ToTargets};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::target::BoolTarget,
    plonk::{circuit_data::CircuitConfig, config::PoseidonGoldilocksConfig},
};

use crate::common_targets::PubkeyTarget;

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyCommitmentMapperFLTarget {
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub pubkey: PubkeyTarget,

    #[target(in)]
    pub is_real: BoolTarget,

    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256: Sha256Target,

    #[target(out)]
    pub poseidon: HashOutTarget,
}

pub struct PubkeyCommitmentMapperFL;

impl Circuit for PubkeyCommitmentMapperFL {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = PubkeyCommitmentMapperFLTarget;

    fn define(
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<Self::F, { Self::D }>,
        _: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        Self::Target {
            pubkey: input.pubkey,
            is_real: input.is_real,
            sha256: sha256_or_zeroes(builder, &input.pubkey, input.is_real),
            poseidon: poseidon_or_zeroes(builder, input.pubkey.to_targets(), input.is_real),
        }
    }
}
