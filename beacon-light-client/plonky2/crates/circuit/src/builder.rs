use std::ops::{Deref, DerefMut};

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder as Plonky2CircuitBuilder, circuit_data::CircuitConfig,
    },
};

pub struct CircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    pub inner: Plonky2CircuitBuilder<F, D>,
    pub targets: Vec<Target>,
}

impl<F: RichField + Extendable<D>, const D: usize> Deref for CircuitBuilder<F, D> {
    type Target = Plonky2CircuitBuilder<F, D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<F: RichField + Extendable<D>, const D: usize> DerefMut for CircuitBuilder<F, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilder<F, D> {
    pub fn new(config: CircuitConfig) -> Self {
        let builder = Plonky2CircuitBuilder::new(config);
        Self {
            inner: builder,
            targets: Vec::new(),
        }
    }
}
