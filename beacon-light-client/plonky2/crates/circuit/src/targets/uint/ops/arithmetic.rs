use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField,
    plonk::circuit_builder::CircuitBuilder,
};

pub trait Add<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn add(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Sub<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn sub(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
}

pub trait Mul<F: RichField + Extendable<D>, const D: usize, Rhs = Self> {
    type Output;

    fn mul(self, rhs: Rhs, builder: &mut CircuitBuilder<F, D>) -> Self::Output;
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
