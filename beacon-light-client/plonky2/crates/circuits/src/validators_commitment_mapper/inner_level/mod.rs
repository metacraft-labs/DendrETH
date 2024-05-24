use circuit::{Circuit, CircuitOutputTarget};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
    },
};

use crate::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    utils::hashing::{poseidon::poseidon_pair, sha256::sha256_pair},
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
};

pub struct ValidatorsCommitmentMapperInnerLevel {}

impl Circuit for ValidatorsCommitmentMapperInnerLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BasicRecursiveInnerCircuitTarget;

    type Params = CircuitData<Self::F, Self::C, { Self::D }>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        circuit_data: &Self::Params,
    ) -> Self::Target {
        let verifier_circuit_target = VerifierCircuitTarget {
            constants_sigmas_cap: builder
                .constant_merkle_cap(&circuit_data.verifier_only.constants_sigmas_cap),
            circuit_digest: builder.constant_hash(circuit_data.verifier_only.circuit_digest),
        };

        let proof1 = builder.add_virtual_proof_with_pis(&circuit_data.common);
        let proof2 = builder.add_virtual_proof_with_pis(&circuit_data.common);

        builder.verify_proof::<Self::C>(&proof1, &verifier_circuit_target, &circuit_data.common);
        builder.verify_proof::<Self::C>(&proof2, &verifier_circuit_target, &circuit_data.common);

        let l_input =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(&proof1.public_inputs);

        let r_input =
            ValidatorsCommitmentMapperFirstLevel::read_public_inputs_target(&proof2.public_inputs);

        let sha256_hash_tree_root = sha256_pair(
            builder,
            &l_input.sha256_hash_tree_root,
            &r_input.sha256_hash_tree_root,
        );

        let poseidon_hash_tree_root = poseidon_pair(
            builder,
            l_input.poseidon_hash_tree_root,
            r_input.poseidon_hash_tree_root,
        );

        let output_target = CircuitOutputTarget::<ValidatorsCommitmentMapperFirstLevel> {
            sha256_hash_tree_root,
            poseidon_hash_tree_root,
        };

        output_target.register_public_inputs(builder);

        Self::Target {
            proof1,
            proof2,
            verifier_circuit_target,
        }
    }
}
