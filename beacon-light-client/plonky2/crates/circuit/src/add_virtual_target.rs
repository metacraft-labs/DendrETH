use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::{circuit_builder::CircuitBuilder, proof::ProofWithPublicInputsTarget},
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

pub trait AddVirtualTarget {
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

impl<const N: usize> AddVirtualTarget for ProofWithPublicInputsTarget<N> {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        _builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        panic!("Recursive proofs are not supported by AddVirtualTarget")
    }
}

impl AddVirtualTarget for BigUintTarget {
    fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        builder.add_virtual_biguint_target(2)
    }
}
