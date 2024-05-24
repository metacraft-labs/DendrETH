use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
        poseidon::PoseidonHash,
    },
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

pub struct IsValidMerkleBranchTargetsPoseidon {
    pub leaf: HashOutTarget,
    pub branch: Vec<HashOutTarget>,
    pub index: Target,
    pub root: HashOutTarget,
}

pub struct IsValidMerkleBranchTargetsPoseidonResult {
    pub leaf: HashOutTarget,
    pub branch: Vec<HashOutTarget>,
    pub index: Target,
    pub root: HashOutTarget,
    pub is_valid: BoolTarget,
}

pub fn is_valid_merkle_branch_poseidon_result<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    depth: usize,
) -> IsValidMerkleBranchTargetsPoseidonResult {
    let index = builder.add_virtual_target();

    let leaf: HashOutTarget = builder.add_virtual_hash();

    let branch: Vec<HashOutTarget> = (0..depth).map(|_| builder.add_virtual_hash()).collect();

    let root: HashOutTarget = builder.add_virtual_hash();

    let indexes = builder.split_le(index, depth + 1);

    let mut hashers: Vec<HashOutTarget> = Vec::new();

    for i in 0..depth {
        let current: HashOutTarget = if i == 0 { leaf } else { hashers[i - 1] };

        for j in 0..NUM_HASH_OUT_ELTS {
            let el1 = builder._if(indexes[i], branch[i].elements[j], current.elements[j]);
            let el2 = builder._if(indexes[i], current.elements[j], branch[i].elements[j]);

            hashers.push(builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![el1, el2]));
        }
    }

    let mut is_valid = builder._true();

    for i in 0..NUM_HASH_OUT_ELTS {
        let is_equal = builder.is_equal(hashers[depth - 1].elements[i], root.elements[i]);
        is_valid = builder.and(is_valid, is_equal);
    }

    IsValidMerkleBranchTargetsPoseidonResult {
        leaf,
        branch,
        index,
        root,
        is_valid,
    }
}

pub fn is_valid_merkle_branch_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    depth: usize,
) -> IsValidMerkleBranchTargetsPoseidon {
    let is_valid_merkle_branch_result_targets =
        is_valid_merkle_branch_poseidon_result(builder, depth);

    let _true = builder._true();

    builder.connect(
        is_valid_merkle_branch_result_targets.is_valid.target,
        _true.target,
    );

    IsValidMerkleBranchTargetsPoseidon {
        leaf: is_valid_merkle_branch_result_targets.leaf,
        branch: is_valid_merkle_branch_result_targets.branch,
        index: is_valid_merkle_branch_result_targets.index,
        root: is_valid_merkle_branch_result_targets.root,
    }
}
