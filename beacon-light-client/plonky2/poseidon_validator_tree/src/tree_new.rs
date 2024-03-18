use std::collections::HashMap;

use crate::tree::combine_two_hash_n_to_hash_no_pad;
use plonky2::{field::{goldilocks_field::GoldilocksField, types::Field}, hash::{hash_types::HashOut, hashing::hash_n_to_hash_no_pad, poseidon::PoseidonPermutation}};

pub const MAX_DEPTH: usize = 3;

pub fn gindex_from_validator_index(index: u64, depth: u32) -> u64 {
    return 2u64.pow(depth - 1) - 1 + index;
}

pub fn zero_hashes() -> Vec<HashOut<GoldilocksField>> {

    const ZERO_HASHES_MAX_INDEX: usize = 41;

    let mut hashes = vec![
        hash_n_to_hash_no_pad::<
            GoldilocksField,
            PoseidonPermutation<GoldilocksField>,>(&[GoldilocksField::from_canonical_u8(0); 32]);
        ZERO_HASHES_MAX_INDEX + 1
    ];

    for i in 0..ZERO_HASHES_MAX_INDEX {
        hashes[i + 1] = combine_two_hash_n_to_hash_no_pad::<GoldilocksField, 2>(hashes[i], hashes[i]);
    }

    hashes
}


pub fn compute_merkle_hash_tree(
    leaves: &[HashOut<GoldilocksField>],
    leave_indices: &[usize],
) -> HashMap<u64, HashOut<GoldilocksField>>
{    
    assert!(leave_indices.len() == leaves.len());

    let zero_hashes = zero_hashes();
    let mut validator_map = HashMap::new();

    for i in 0..leaves.len() {
        validator_map.insert(gindex_from_validator_index(leave_indices[i] as u64, MAX_DEPTH as u32), leaves[i]);
    }

    for cur_depth in (0..MAX_DEPTH).rev() {        
        println!("Cur. Depth: {}", cur_depth);
        compute_hashes_at_depth(
            cur_depth as u32,
            &mut validator_map,
            &zero_hashes
        );
    }

    validator_map
}

fn left_right_child_from_hashmap(
    validator_map: &mut HashMap<u64, HashOut<GoldilocksField>>,
    zero_hashes: &Vec<HashOut<GoldilocksField>>,
    index: u32,
    depth: u32
) -> (HashOut<GoldilocksField>, HashOut<GoldilocksField>) {

    let left_key = gindex_from_validator_index(2*index as u64, depth + 1);
    let right_key = gindex_from_validator_index((2*index + 1) as u64, depth + 1);

    let left_elem = 
        if validator_map.contains_key(&left_key) {validator_map[&left_key]} else {zero_hashes[MAX_DEPTH - (depth + 1) as usize]};
    let right_elem = 
        if validator_map.contains_key(&right_key) {validator_map[&right_key]} else {zero_hashes[MAX_DEPTH - (depth + 1) as usize]};

    (left_elem,right_elem)
}

pub fn compute_hashes_at_depth(
    depth: u32,
    validator_map: &mut HashMap<u64, HashOut<GoldilocksField>>,
    zero_hashes: &Vec<HashOut<GoldilocksField>>
){

    for cur_index in 0..depth {

        let (left, right) = left_right_child_from_hashmap(
            validator_map,
            zero_hashes,
            cur_index,
            depth
        );

        let cur_hash 
            = combine_two_hash_n_to_hash_no_pad::<GoldilocksField, 2>(
                left,
                right
            );
        let gindex = gindex_from_validator_index(cur_index as u64, depth);
        validator_map.insert(gindex,cur_hash);
    }
}

pub fn get_validator_proof(
    validator_gindex: u64,
    validator_map: &HashMap<u64, HashOut<GoldilocksField>>
    ) -> Vec<HashOut<GoldilocksField>> {

    let zero_hashes = zero_hashes();

    let mut proof = Vec::with_capacity(MAX_DEPTH as usize);
    let mut gindex = validator_gindex;
    let mut cur_depth = 0;

    while gindex != 0 {
        let sibling_gindex = if gindex % 2 == 0 { gindex -1} else {gindex + 1};

        let element_to_push = if validator_map.contains_key(&sibling_gindex) {
            validator_map[&sibling_gindex]
        } else {
            zero_hashes[cur_depth]
        };
        proof.push(element_to_push);

        cur_depth += 1;
        gindex = (gindex -1) / 2;
    }

    assert!(proof.len() + 1 == MAX_DEPTH as usize);

    proof
}

pub fn prove_validator_membership(
    validator_hash: HashOut<GoldilocksField>,
    validator_proof: Vec<HashOut<GoldilocksField>>,
    merkle_tree_root: HashOut<GoldilocksField>,
    validator_gindex: usize
) {
    let mut path = [false; MAX_DEPTH - 1];
    let mut hash = validator_hash;

    for i in 0..(MAX_DEPTH - 1) {
        path[i] = (validator_gindex & (1 << i)) == 0;
    }

    for idx in 0..(MAX_DEPTH - 1) {
        if path[idx] == true {
            hash = combine_two_hash_n_to_hash_no_pad::<GoldilocksField,2>(
                hash,
                validator_proof[idx]
            );
        }
        else {
            hash = combine_two_hash_n_to_hash_no_pad::<GoldilocksField,2>(
                validator_proof[idx],
                hash
            );
        }

    }

    assert!(merkle_tree_root == hash);
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use plonky2::field::goldilocks_field::GoldilocksField;

    use crate::objects::Validator;

    const D: usize = 2;
    type F = GoldilocksField;

    const file_path_attestations: &str = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data.json";

    const validators_raw: Vec<Validator> = read_validator_data(file_path_attestations);

    // let mut poseidon_validator_obj_vec = Vec::with_capacity(validators_raw.len());
    let mut validators_hashed: Vec<HashOut<GoldilocksField>> = Vec::with_capacity(validators_raw.len());

    #[test]
    fn test_1() {

    }

}
