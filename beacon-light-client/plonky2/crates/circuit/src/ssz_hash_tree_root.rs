use itertools::Itertools;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::targets::uint::Uint64Target;

pub trait SSZHashTreeRoot<F: RichField + Extendable<D>, const D: usize> {
    fn ssz_hash_tree_root(self, builder: &mut CircuitBuilder<F, D>) -> [BoolTarget; 256];
}

impl<F: RichField + Extendable<D>, const D: usize> SSZHashTreeRoot<F, D> for Uint64Target {
    fn ssz_hash_tree_root(self, builder: &mut CircuitBuilder<F, D>) -> [BoolTarget; 256] {
        let _false = builder._false();

        self.to_le_bytes(builder)
            .into_iter()
            .pad_using(256, |_| _false)
            .collect_vec()
            .try_into()
            .unwrap()
    }
}
