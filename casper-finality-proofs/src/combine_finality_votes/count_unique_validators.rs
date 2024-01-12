use crate::constants::VALIDATOR_INDICES_IN_SPLIT;
use crate::utils::plonky2x_extensions::assert_is_true;

use plonky2x::prelude::{ArrayVariable, CircuitBuilder, PlonkParameters, U64Variable};

#[derive(Debug, Clone)]
pub struct CountUniqueValidators;

impl CountUniqueValidators {
    pub fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let sigma = builder.read::<U64Variable>();
        let mut validator_indices = vec![];
        for _ in 0..VALIDATOR_INDICES_IN_SPLIT {
            let cur = builder.read::<U64Variable>();
            validator_indices.push(cur);
        }
        
        let _zero: U64Variable = builder.zero();
        let _one: U64Variable = builder.one();

        let mut total_unique: U64Variable = builder.one();
        let mut is_aligned = builder._true();

        let mut private_accumulator = builder.mul(sigma, validator_indices[0]);
        for i in 1..VALIDATOR_INDICES_IN_SPLIT {
            let validator_sigma_mul = builder.mul(validator_indices[i], sigma);
            private_accumulator = builder.add(private_accumulator, validator_sigma_mul);

            is_aligned = builder.lte(validator_indices[i - 1], validator_indices[i]);

            let should_count_pred = builder.lt(validator_indices[i - 1], validator_indices[i]);
            let value_to_add = builder.select(should_count_pred, _one, _zero);
            total_unique = builder.add(total_unique, value_to_add);
        }

        assert_is_true(builder, is_aligned);

        builder.write(total_unique);
        builder.write(private_accumulator);
        builder.write(validator_indices[0]);
        builder.write(validator_indices[VALIDATOR_INDICES_IN_SPLIT - 1]);
    }
}
