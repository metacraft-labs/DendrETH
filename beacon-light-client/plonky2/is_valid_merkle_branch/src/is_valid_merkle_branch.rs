use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use plonky2_sha256::circuit::make_circuits;

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

    let leaf: [BoolTarget; 256] = [builder.add_virtual_bool_target_safe(); 256];
    let mut branch: Vec<[BoolTarget; 256]> = Vec::new();

    for _i in 0..depth {
        branch.push([builder.add_virtual_bool_target_safe(); 256]);
    }

    let root: [BoolTarget; 256] = [builder.add_virtual_bool_target_safe(); 256];

    let indexes = builder.split_le(index, depth);

    let mut hashers = Vec::new();
    hashers.push(make_circuits(builder, 512));

    for i in 0..depth {
        hashers.push(make_circuits(builder, 512));

        let current: [BoolTarget; 256] = if i == 0 {
            leaf
        } else {
            hashers[i - 1].digest.clone().try_into().unwrap()
        };

        for j in 0..256 {
            let el1 = builder._if(indexes[i], branch[i][j].target, current[j].target);
            builder.connect(hashers[i].message[i].target, el1);

            let el2 = builder._if(indexes[i], current[j].target, branch[i][j].target);
            builder.connect(hashers[i].message[i + 256].target, el2);
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
