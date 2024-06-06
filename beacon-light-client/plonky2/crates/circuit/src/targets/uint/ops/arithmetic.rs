use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

pub trait Add<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn add(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait CircuitBuilderUnsignedInteger<F: RichField + Extendable<D>, const D: usize> {
    fn add_uint<Lhs, Rhs>(&mut self, lhs: Lhs, rhs: Rhs) -> <Lhs as Add<F, D, Rhs>>::Output
    where
        Lhs: Add<F, D, Rhs>;
}

pub trait Sub<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn sub(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Mul<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn mul(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Neg<F: RichField + Extendable<D>, const D: usize> {
    type Output;

    fn neg(self, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Div<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn div(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Rem<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn rem(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Zero<F: RichField + Extendable<D>, const D: usize> {
    fn zero(builder: &mut CircuitBuilder<F, D>) -> Self;
}

pub trait One<F: RichField + Extendable<D>, const D: usize> {
    fn one(builder: &mut CircuitBuilder<F, D>) -> Self;
}

pub trait LessThanOrEqual<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    #[must_use]
    fn lte(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> BoolTarget;
}
