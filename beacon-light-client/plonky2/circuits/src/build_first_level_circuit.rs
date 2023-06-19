use plonky2::{
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

use crate::validator_commitment::{validator_commitment, ValidatorCommitment};

pub fn build_first_level_circuit() -> (
    ValidatorCommitment,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let validator_commitment_result = validator_commitment(&mut builder);

    builder.register_public_inputs(&validator_commitment_result.poseidon_hash_tree_root.elements);
    builder.register_public_inputs(
        &validator_commitment_result
            .sha256_hash_tree_root
            .map(|x| x.target),
    );

    let data = builder.build::<C>();

    (validator_commitment_result, data)
}
