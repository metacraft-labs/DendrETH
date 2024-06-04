use crate::{
    common_targets::{BasicRecursiveInnerCircuitTarget, Sha256Target},
    serializers::serde_bool_array_to_hex_string,
    utils::circuit::{
        hashing::{poseidon::poseidon_pair, sha256::sha256_pair},
        verify_proof,
    },
};
use circuit::{Circuit, CircuitOutputTarget};
use circuit_derive::CircuitTarget;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    plonk::{
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};

use super::first_level::PubkeyCommitmentMapperFL;

#[derive(CircuitTarget)]
pub struct PubkeyCommitmentMapperILTarget {
    #[target(out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub sha256: Sha256Target,

    #[target(out)]
    pub poseidon: HashOutTarget,
}

pub struct PubkeyCommitmentMapperIL;

impl Circuit for PubkeyCommitmentMapperIL {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BasicRecursiveInnerCircuitTarget;

    type Params = CircuitData<Self::F, Self::C, { Self::D }>;

    fn define(
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<Self::F, { Self::D }>,
        circuit_data: &Self::Params,
    ) -> Self::Target {
        let left = verify_proof(builder, &circuit_data);
        let right = verify_proof(builder, &circuit_data);

        let left_pis = PubkeyCommitmentMapperFL::read_public_inputs_target(&left.public_inputs);
        let right_pis = PubkeyCommitmentMapperFL::read_public_inputs_target(&right.public_inputs);

        let output_target = CircuitOutputTarget::<PubkeyCommitmentMapperFL> {
            sha256: sha256_pair(builder, &left_pis.sha256, &right_pis.sha256),
            poseidon: poseidon_pair(builder, left_pis.poseidon, right_pis.poseidon),
        };

        output_target.register_public_inputs(builder);

        Self::Target {
            proof1: left,
            proof2: right,
        }
    }
}
