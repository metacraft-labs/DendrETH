use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

pub trait LessThanOrEqual<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    #[must_use]
    fn lte(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget;
}

pub trait EqualTo<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    #[must_use]
    fn equal_to(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget;
}

pub trait Comparison<F: RichField + Extendable<D>, const D: usize, Rhs = Self>:
    LessThanOrEqual<F, D, Rhs> + EqualTo<F, D, Rhs>
{
    #[must_use]
    fn lt(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget
    where
        Self: Sized + Copy,
        Rhs: Copy,
    {
        let are_equal = self.equal_to(rhs, builder);
        let are_not_equal = builder.not(are_equal);
        let lte = self.lte(rhs, builder);
        builder.and(lte, are_not_equal)
    }

    #[must_use]
    fn gt(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget
    where
        Self: Sized + Copy,
    {
        let lte = self.lte(rhs, builder);
        builder.not(lte)
    }

    #[must_use]
    fn gte(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget
    where
        Self: Sized + Copy,
        Rhs: Copy,
    {
        let are_equal = self.equal_to(rhs, builder);
        let gt = self.gt(rhs, builder);
        builder.or(gt, are_equal)
    }

    #[must_use]
    fn not_equal_to(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget
    where
        Self: Sized,
    {
        let are_equal = self.equal_to(rhs, builder);
        builder.not(are_equal)
    }
}
