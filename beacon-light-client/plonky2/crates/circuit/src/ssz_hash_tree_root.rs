use itertools::Itertools;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::targets::uint::Uint64Target;

pub trait SSZHashTreeRoot<F: RichField + Extendable<D>, const D: usize> {
    fn ssz_hash_tree_root(self, builder: &mut CircuitBuilder<F, D>) -> [BoolTarget; 256];
}

impl<F: RichField + Extendable<D>, const D: usize> crate::SSZHashTreeRoot<F, D> for Uint64Target {
    fn ssz_hash_tree_root(self, builder: &mut CircuitBuilder<F, D>) -> [BoolTarget; 256] {
        let mut le_bytes_bits: Vec<BoolTarget> = self
            .limbs
            .into_iter()
            .map(|limb| {
                builder
                    .split_le_base::<2>(limb.0, 32)
                    .into_iter()
                    .map(|target| BoolTarget::new_unsafe(target))
                    .rev()
                    .collect_vec()
            })
            .flatten()
            .collect_vec();
        le_bytes_bits.extend((le_bytes_bits.len()..256).map(|_| builder._false()));
        le_bytes_bits.try_into().unwrap()
    }
}
