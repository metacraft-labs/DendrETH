use std::collections::HashMap;

use num_bigint::BigUint;
use plonky2::{field::{extension::Extendable, goldilocks_field::GoldilocksField, types::Field}, hash::{hash_types::{HashOut, RichField}, hashing::hash_n_to_hash_no_pad, poseidon::PoseidonPermutation}, plonk::config::GenericHashOut};

use super::poseidon_helpers::{read_validator_data, Validator};

pub const MAX_DEPTH: usize = 41;

pub fn parse_validator_data(file_path: &str) -> (Vec<Validator>,Vec<HashOut<GoldilocksField>>) {
    let  validators_raw: Vec<Validator> = read_validator_data(file_path);

    println!("Total number of validators - {}", validators_raw.len());

    let mut validators_hashed: Vec<HashOut<GoldilocksField>> = Vec::with_capacity(validators_raw.len());

    for i in 0..validators_raw.len() {
        let cur_validator_hash = 
            compute_validator_poseidon_hash(validators_raw[i].clone());

        validators_hashed.push(cur_validator_hash);
    }

    (validators_raw, validators_hashed)
}

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

fn hash_bits_arr_in_goldilocks_to_hash_no_pad(
    validator_data: &[bool],
) -> HashOut<GoldilocksField> {
    let validator_data_in_goldilocks: Vec<GoldilocksField> = validator_data
        .iter()
        .map(|x| GoldilocksField::from_bool(*x))
        .collect();

    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(
        validator_data_in_goldilocks.as_slice(),
    )
}

fn hash_biguint_in_goldilocks_to_hash_no_pad(
    validator_data: BigUint,
) -> HashOut<GoldilocksField> {
    let mut validator_data_in_goldilocks = validator_data.to_u32_digits();
    assert!(validator_data_in_goldilocks.len() <= 2);
    validator_data_in_goldilocks.resize(2, 0);
    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
        GoldilocksField::from_canonical_u32(validator_data_in_goldilocks[0]),
        GoldilocksField::from_canonical_u32(validator_data_in_goldilocks[1]),
    ])
}

pub fn compute_poseidon_hash_tree_root(
    leaves_len: usize,
    leaves: Vec<HashOut<GoldilocksField>>,
) -> HashOut<GoldilocksField> {
    let mut hashers: Vec<HashOut<GoldilocksField>> = Vec::new();
    for i in 0..(leaves_len / 2) {
        let goldilocks_leaves = leaves[i * 2]
            .elements
            .iter()
            .copied()
            .chain(leaves[i * 2 + 1].elements.iter().copied())
            .into_iter();
        let goldilocks_leaves_collected: Vec<GoldilocksField> = goldilocks_leaves.collect();
        hashers.push(hash_n_to_hash_no_pad::<
            GoldilocksField,
            PoseidonPermutation<GoldilocksField>,
        >(&goldilocks_leaves_collected));
    }

    let mut k = 0;
    for _ in leaves_len / 2..leaves_len - 1 {
        let goldilocks_leaves = hashers[k * 2]
            .elements
            .iter()
            .copied()
            .chain(hashers[k * 2 + 1].elements.iter().copied());
        let goldilocks_leaves_collected: Vec<GoldilocksField> = goldilocks_leaves.collect();
        hashers.push(hash_n_to_hash_no_pad::<
            GoldilocksField,
            PoseidonPermutation<GoldilocksField>,
        >(&goldilocks_leaves_collected));

        k += 1;
    }

    hashers[leaves_len - 2]
}

pub fn combine_two_hash_n_to_hash_no_pad<F: RichField + Extendable<D>, const D: usize>(
    left: HashOut<GoldilocksField>,
    right: HashOut<GoldilocksField>,
) -> HashOut<GoldilocksField> {
    let left_node_in_goldilocks: Vec<GoldilocksField> = left
        .to_bytes()
        .iter()
        .map(|&x| GoldilocksField::from_canonical_u8(x))
        .collect();

    let right_node_in_goldilocks: Vec<GoldilocksField> = right
        .to_bytes()
        .iter()
        .map(|&x| GoldilocksField::from_canonical_u8(x))
        .collect();

    let combined_nodes: Vec<GoldilocksField> = left_node_in_goldilocks
        .into_iter()
        .chain(right_node_in_goldilocks.into_iter())
        .collect();

    hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&combined_nodes)
}


pub fn compute_validator_poseidon_hash(
    validator: Validator,
) -> HashOut<GoldilocksField> {
    let leaves = vec![
        hash_bits_arr_in_goldilocks_to_hash_no_pad(&validator.pubkey),
        hash_bits_arr_in_goldilocks_to_hash_no_pad(&validator.withdrawal_credentials),
        hash_biguint_in_goldilocks_to_hash_no_pad(validator.effective_balance.clone()),
        hash_n_to_hash_no_pad::<GoldilocksField, PoseidonPermutation<GoldilocksField>>(&[
            GoldilocksField::from_bool(validator.slashed),
        ]),
        hash_biguint_in_goldilocks_to_hash_no_pad(
            validator.activation_eligibility_epoch.clone(),
        ),
        hash_biguint_in_goldilocks_to_hash_no_pad(validator.activation_epoch.clone()),
        hash_biguint_in_goldilocks_to_hash_no_pad(validator.exit_epoch.clone()),
        hash_biguint_in_goldilocks_to_hash_no_pad(validator.withdrawable_epoch.clone()),
    ];
    let poseidon_hash_tree_root =
        compute_poseidon_hash_tree_root(leaves.len(), leaves.clone());

    poseidon_hash_tree_root
}

pub fn compute_merkle_hash_tree(
    leaves: &[HashOut<GoldilocksField>],
) -> HashMap<u64, HashOut<GoldilocksField>>
{    
    let zero_hashes = zero_hashes();
    let mut validator_map = HashMap::new();

    for i in 0..leaves.len() {
        validator_map.insert(gindex_from_validator_index(i as u64, MAX_DEPTH as u32), leaves[i]);
    }

    for cur_depth in (0..MAX_DEPTH).rev() {        
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
    validator_map: &HashMap<u64, HashOut<GoldilocksField>>,
    zero_hashes: &Vec<HashOut<GoldilocksField>>
    ) -> Vec<HashOut<GoldilocksField>> {

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
    validator_index: usize
) {
    let mut path = [false; MAX_DEPTH - 1];
    let mut hash = validator_hash;

    for i in 0..(MAX_DEPTH - 1) {
        path[i] = (validator_index & (1 << i)) == 0;
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
    use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut};

    use crate::utils::poseidon_merkle_tree::{combine_two_hash_n_to_hash_no_pad,compute_merkle_hash_tree, get_validator_proof, gindex_from_validator_index, parse_validator_data, prove_validator_membership, MAX_DEPTH};

    use super::zero_hashes;

    const FILE_PATH_TOY_DATA: &str = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data.json";

    #[test]
    fn test_leftmost() {
        // Change MAX_DEPTH constant = 3

        let (_, validators_hashed) = parse_validator_data(FILE_PATH_TOY_DATA);

        let zero_hashes = zero_hashes();

        let mut validator_map = compute_merkle_hash_tree(
            &validators_hashed, 
        );
    
        let validator_gindex = gindex_from_validator_index(0, MAX_DEPTH as u32);
        let proof = get_validator_proof(validator_gindex, &mut validator_map, &zero_hashes);
    
        let hash = combine_two_hash_n_to_hash_no_pad::<GoldilocksField, 2>(
            validators_hashed[0],
            proof[0]
        );
    
        let hash2 = combine_two_hash_n_to_hash_no_pad::<GoldilocksField, 2>(
            hash,
            proof[1]
        );
        println!("Manualy Computed Root: {:?}\nGiven Root: {:?}\n", hash2, validator_map[&0]);
        assert!(hash2 == validator_map[&0]);
    
        prove_validator_membership(
            validators_hashed[0],
            proof.clone(),
            validator_map[&0],
            0,
        );
    }

    #[test]
    fn test_with_missing_hash() {
        // Change MAX_DEPTH constant = 3

        let (_, validators_hashed) = parse_validator_data(FILE_PATH_TOY_DATA);

        let zeroes = zero_hashes();

        let mut validators_with_missing: Vec<HashOut<GoldilocksField>> = 
            validators_hashed;

        validators_with_missing[3] = zeroes[0];

        let validator_map = compute_merkle_hash_tree(
            &validators_with_missing, 
        );
    
        for i in 0..validators_with_missing.len(){
    
            let validator_gindex = gindex_from_validator_index(i as u64, MAX_DEPTH as u32);
            let proof = get_validator_proof(validator_gindex, &validator_map, &zeroes);
            println!("\nValidator-{} proof: {:?}", i,proof);
    
            prove_validator_membership(
                validators_with_missing[i],
                proof.clone(),
                validator_map[&0],
                i,
            );
            println!("Proof for validator-{} passed!",i);
        }
    }

    #[test]
    fn test_with_depth_40() {
        // set MAX_DEPTH = 40
        
        let (_, validators_hashed) = parse_validator_data(FILE_PATH_TOY_DATA);

        let validator_map = compute_merkle_hash_tree(
            &validators_hashed, 
        );
    
        let zero_hashes = zero_hashes();

        for i in 0..validators_hashed.len(){
    
            let validator_gindex = gindex_from_validator_index(i as u64, MAX_DEPTH as u32);
            let proof = get_validator_proof(validator_gindex, &validator_map, &zero_hashes);
            println!("\nValidator-{} proof: {:?}", i,proof);
            println!("Proof Len. {}",proof.len());
            prove_validator_membership(
                validators_hashed[i],
                proof.clone(),
                validator_map[&0],
                i,
            );
            println!("Proof for validator-{} passed!", i);
        }
    }
}
