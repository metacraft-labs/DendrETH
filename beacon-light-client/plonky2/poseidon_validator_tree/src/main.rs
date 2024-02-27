use std::{collections::HashMap, time::{Duration, Instant}};
use num_bigint::BigUint;
use plonky2::{field::{goldilocks_field::GoldilocksField, types::PrimeField64}, hash::hash_types::HashOut};
use poseidon_validator_tree::{
    objects::{Validator, ValidatorPoseidonDataOutput},
    parse_validators::{binary_to_hex, read_validator_data}, 
    tree::{compute_poseidon_hash_tree_root, compute_validator_poseidon_hash, compute_validators_merkle_proof, return_every_validator_hash, MerkleTree}
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PoseidonData {
    validators: Vec<ValidatorPoseidonDataOutput>,
    // poseidon_root: HashOut<GoldilocksField>,
}

fn biguint_to_u64_unsafe(x: BigUint) -> u64 {
    let result = x.to_u64_digits();
    

    if result.is_empty() {
        0
    }
    else {
        result[0]
    }

}

pub fn main() {

    const D: usize = 2;
    type F = GoldilocksField;

    //TODO: validator count is - 911215, hashed validators are - 911203, why?
    let file_path_attestations = 
                "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400.json";
    let file_path_attestations_out = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400_poseidon.json";

    let  validators_raw: Vec<Validator> = read_validator_data(file_path_attestations);

    let mut poseidon_validator_obj_vec = Vec::with_capacity(validators_raw.len());
    let mut validators_hashed: Vec<HashOut<GoldilocksField>> = Vec::with_capacity(validators_raw.len());

    const DEPTH: usize = 40;

    let mut start_time = Instant::now();

    for i in 0..validators_raw.len() {

        if i%100_000==0 {
            println!("Computing {}-th validator_hash..", i);
        }

        let cur_validator_hash = compute_validator_poseidon_hash::<F,D>(validators_raw[i].clone());
        validators_hashed.push(cur_validator_hash);
    }

    let num_validators = validators_hashed.len();
    
    let mut end_time = Instant::now();
    let mut duration = end_time - start_time;

    println!("Casting Validators took: {}", duration.as_millis());
    start_time = Instant::now();
    
    let merkle_tree = MerkleTree::new::<F, D>(&validators_hashed, DEPTH);

    // let exponent = u64::pow(2,DEPTH as u32);
    // let poseidon_hash_tree_root 
    //     = compute_poseidon_hash_tree_root::<F, D>(validators_hashed.len() as usize,validators_hashed.clone());
    
    // println!("ROOT: {:?}", poseidon_hash_tree_root);

    // end_time = Instant::now();
    // duration = end_time - start_time;
    // println!("HashTreeRoot Generation Took: {}", duration.as_millis());
    // start_time = Instant::now();
    
    for i in 0..num_validators {

        let (_leaf, proof) = merkle_tree.generate_proof::<F, D>(i, DEPTH).unwrap();

        if i%10_000 == 0 {
            println!("On {}-th validator hash..", i);
        }

        poseidon_validator_obj_vec.push(
        ValidatorPoseidonDataOutput {
                trusted: validators_raw[i].trusted,
                validator_index: validators_raw[i].validator_index,

                activation_eligibility_epoch: biguint_to_u64_unsafe(validators_raw[i].activation_eligibility_epoch.clone()),
                activation_epoch: biguint_to_u64_unsafe(validators_raw[i].activation_epoch.clone()),
                effective_balance: biguint_to_u64_unsafe(validators_raw[i].effective_balance.clone()),
                exit_epoch: biguint_to_u64_unsafe(validators_raw[i].exit_epoch.clone()),
                pubkey: binary_to_hex(validators_raw[i].pubkey.as_slice()),
                slashed: validators_raw[i].slashed,
                withdrawable_epoch: biguint_to_u64_unsafe(validators_raw[i].withdrawable_epoch.clone()),
                withdrawal_credentials: binary_to_hex(validators_raw[i].withdrawal_credentials.as_slice()),
                validator_poseidon_hash: validators_hashed[i],
                validator_poseidon_proof: proof
            }
        );
    }

    // end_time = Instant::now();
    // duration = end_time - start_time;

    // println!("Proof Generation Took: {}", duration.as_millis());

    let poseidon_data = PoseidonData {
        validators: poseidon_validator_obj_vec,
        // poseidon_root: poseidon_hash_tree_root
    };

    let mut data_map = HashMap::new();
    data_map.insert("data", poseidon_data);


    let json_poseidon_out = serde_json::to_string(&data_map).expect("Failed to serialize");
    std::fs::write(file_path_attestations_out, json_poseidon_out).expect("Failed to write file");

} 
