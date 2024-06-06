use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

pub trait LessThanOrEqual<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    #[must_use]
    fn lte(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget;
}
