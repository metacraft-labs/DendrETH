use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::public_inputs::target_reader::PublicInputsTargetReadable;

pub trait AddVirtualTarget: PublicInputsTargetReadable {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self;
}

impl AddVirtualTarget for Target {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        builder.add_virtual_target()
    }
}

impl AddVirtualTarget for BoolTarget {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        builder.add_virtual_bool_target_safe()
    }
}

impl<T: AddVirtualTarget + std::fmt::Debug, const N: usize> AddVirtualTarget for [T; N] {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        [(); N].map(|_| T::add_virtual_target(builder))
    }
}

impl AddVirtualTarget for HashOutTarget {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        builder.add_virtual_hash()
    }
}
