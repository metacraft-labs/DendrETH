use std::collections::HashMap;

use crate::tree::combine_two_hash_n_to_hash_no_pad;
use lazy_static::lazy_static;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut};

// lazy_static! {
//     static ref SYNTHETIC_ZERO_NODES: Vec<HashOut<GoldilocksField>> = {
//         const MAX_TREE_DEPTH: usize = 32;
//         (0..=MAX_TREE_DEPTH).map(MerkleTree::Zero).collect()
//     };
// }

pub fn gindex_from_validator_index(index: u64, depth: u32) -> u64 {
    return 2u64.pow(depth) - 1 + index;
}

pub fn compute_merkle_hash_tree_root(leaves: &[HashOut<GoldilocksField>], depth: u32) -> Vec<HashOut<GoldilocksField>> {
    let mut all_hashes = Vec::with_capacity(2usize.pow(depth));
    //TODO: Add synthetic zeroes
    for leaf in leaves {1234
        
        all_hashes.push(*leaf);
    }

    let mut validator_index_cutoff = 0; 
    for cur_depth in (0..depth).rev() {
        let step_size = 2usize.pow(cur_depth);
        
        let computed_hashes = compute_hashes_at_depth(
            &all_hashes[validator_index_cutoff..validator_index_cutoff + step_size],
            cur_depth
        );
        all_hashes.extend_from_slice(&computed_hashes);

        validator_index_cutoff += step_size;
    }

    all_hashes
}

pub fn compute_hashes_at_depth(leaves: &[HashOut<GoldilocksField>], depth: u32) -> Vec<HashOut<GoldilocksField>> {

    println!("{}",leaves.len());

    let mut all_hashes =Vec::with_capacity(2usize.pow(depth));
    for cur_index in 0..depth{
        
        let cur_hash 
            = combine_two_hash_n_to_hash_no_pad::<GoldilocksField, 2>(
                leaves[2*cur_index as usize],
                leaves[(2*cur_index + 1) as usize]
            );
        // all_hashes[cur_index as usize] = cur_hash;
        all_hashes.push(cur_hash);
    }

    all_hashes
}
