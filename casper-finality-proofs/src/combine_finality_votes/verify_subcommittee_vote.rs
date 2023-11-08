use plonky2x::prelude::Field;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{ArrayVariable, BoolVariable, CircuitBuilder, PlonkParameters, Variable},
};

pub const BITMASK_SIZE: usize = 2_000_000;

#[derive(Debug, Clone)]
pub struct VerifySubcommitteeVote;

impl Circuit for VerifySubcommitteeVote {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>)
    where
        <<L as PlonkParameters<D>>::Config as plonky2::plonk::config::GenericConfig<D>>::Hasher:
            plonky2::plonk::config::AlgebraicHasher<<L as PlonkParameters<D>>::Field>,
    {
        let set_bit = builder.read::<Variable>();

        let source = builder.zero();
        let target = builder.one();
        let voted_count =
            builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(1));

        let mut bitmask_data = vec![builder._false(); BITMASK_SIZE];
        for i in 0..BITMASK_SIZE {
            let idx = builder.constant(<L as PlonkParameters<D>>::Field::from_canonical_usize(i));
            let should_set_bit_pred = builder.is_equal(idx, set_bit);
            let _true = builder._true();
            bitmask_data[i] = builder.select(should_set_bit_pred, _true, bitmask_data[i]);
        }

        let bitmask: ArrayVariable<BoolVariable, BITMASK_SIZE> = ArrayVariable::new(bitmask_data);

        builder.write::<Variable>(source);
        builder.write::<Variable>(target);
        builder.write::<Variable>(voted_count);
        builder.write::<ArrayVariable<BoolVariable, BITMASK_SIZE>>(bitmask);
    }
}
