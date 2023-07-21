use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::sha256::make_circuits;
use crate::utils::create_bool_target_array;

pub struct IsValidMerkleBranchTargets {
    pub leaf: [BoolTarget; 256],
    pub branch: Vec<[BoolTarget; 256]>,
    pub index: Target,
    pub root: [BoolTarget; 256],
}

pub fn is_valid_merkle_branch<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    depth: usize,
) -> IsValidMerkleBranchTargets {
    let index = builder.add_virtual_target();

    let leaf: [BoolTarget; 256] = create_bool_target_array(builder);

    let branch: Vec<[BoolTarget; 256]> = (0..depth)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let root: [BoolTarget; 256] = create_bool_target_array(builder);

    let indexes = builder.split_le(index, depth + 1);

    let mut hashers = Vec::new();

    for i in 0..depth {
        hashers.push(make_circuits(builder, 512));
        let current: [BoolTarget; 256] = if i == 0 {
            leaf
        } else {
            hashers[i - 1].digest.clone().try_into().unwrap()
        };

        for j in 0..256 {
            let el1 = builder._if(indexes[i], branch[i][j].target, current[j].target);
            builder.connect(hashers[i].message[j].target, el1);

            let el2 = builder._if(indexes[i], current[j].target, branch[i][j].target);
            builder.connect(hashers[i].message[j + 256].target, el2);
        }
    }

    for i in 0..256 {
        builder.connect(hashers[depth - 1].digest[i].target, root[i].target)
    }

    IsValidMerkleBranchTargets {
        leaf: leaf,
        branch: branch,
        index: index,
        root: root,
    }
}
