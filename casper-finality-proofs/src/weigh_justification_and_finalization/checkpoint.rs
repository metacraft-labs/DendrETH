use crate::types::{Epoch, Root};
use plonky2x::prelude::RichField;
use plonky2x::{
    frontend::vars::SSZVariable,
    prelude::{Bytes32Variable, CircuitBuilder, CircuitVariable, PlonkParameters, Variable},
};

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(CheckpointValue)]
pub struct CheckpointVariable {
    pub epoch: Epoch,
    pub root: Root,
}

impl SSZVariable for CheckpointVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let epoch_leaf = builder.ssz_hash_tree_root(self.epoch);
        let root_leaf = builder.ssz_hash_tree_root(self.root);
        builder.curta_sha256_pair(epoch_leaf, root_leaf)
    }
}
