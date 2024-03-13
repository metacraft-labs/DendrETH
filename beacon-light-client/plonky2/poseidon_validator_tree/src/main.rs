use std::{collections::HashMap, time::{Duration, Instant}};
use num_bigint::BigUint;
use plonky2::{field::{goldilocks_field::GoldilocksField, types::PrimeField64}, hash::hash_types::HashOut};
use poseidon_validator_tree::{
    objects::{Validator, ValidatorPoseidonDataOutput},
    parse_validators::{binary_to_hex, read_validator_data}, 
    tree::{combine_two_hash_n_to_hash_no_pad, compute_poseidon_hash_tree_root, compute_validator_poseidon_hash, MerkleTree}, tree_new::compute_merkle_hash_tree_root, 
    // tree_new::compute_tree_from_leaves
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PoseidonData {
    validators: Vec<ValidatorPoseidonDataOutput>,
    poseidon_root: HashOut<GoldilocksField>,
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

pub fn prove_validator_membership( // Function is correct
    mut validator: ValidatorPoseidonDataOutput,
    validator_index: usize,
    merkle_hash_tree_root: HashOut<GoldilocksField>,
    depth: usize) {

        // let gindex = validator_index + 2usize.pow(depth as u32) - 1;
        let gindex = validator_index;
        let mut bool_array = [false; 32];
        let mut hash = validator.validator_poseidon_hash;

        for i in 0..32 {
            bool_array[i] = (gindex & (1 << i)) != 0;
        }

        println!("bool_array: {:?}\n", bool_array);

        for idx in 0..depth {
            println!("CUR_HASH: {:?}\nCUR_PROOF: {:?}\n", hash, validator.validator_poseidon_proof[idx]);
            if bool_array[idx] == true { // Right
                hash = combine_two_hash_n_to_hash_no_pad::<GoldilocksField,2>(validator.validator_poseidon_proof[idx],hash);
            }
            else { // Left
                println!("Hi");
                hash = combine_two_hash_n_to_hash_no_pad::<GoldilocksField,2>(hash,validator.validator_poseidon_proof[idx]);
            }

        }

        println!("hash: {:?}\nroot: {:?}\n",hash,merkle_hash_tree_root);
}

pub fn main() {

    // Run some tests
    const D: usize = 2;
    type F = GoldilocksField;

    let file_path_attestations = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data.json";
    let file_path_attestations_out = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data_out.json";

    let  validators_raw: Vec<Validator> = read_validator_data(file_path_attestations);

    let mut poseidon_validator_obj_vec = Vec::with_capacity(validators_raw.len());
    let mut validators_hashed: Vec<HashOut<GoldilocksField>> = Vec::with_capacity(validators_raw.len());

    const DEPTH: usize = 3;

    for i in 0..validators_raw.len() {

        println!("Computing {}-th validator_hash..", i);

        let cur_validator_hash = compute_validator_poseidon_hash::<F,D>(validators_raw[i].clone());
        validators_hashed.push(cur_validator_hash);
    }

    let result = compute_merkle_hash_tree_root(&validators_hashed, DEPTH as u32);

    println!("\n RESULT \n {:?}",result.len());
    println!("\n RESULT \n {:?}",result);

    let merkle_tree = MerkleTree::new::<F, D>(&validators_hashed, DEPTH);
    
    let num_validators = validators_hashed.len();

    println!("Number of validators: {}\n", num_validators);
    for i in 0..num_validators {

        let (_leaf, proof) = 
            merkle_tree.generate_proof::<F, D>(
                // validators_raw[i].validator_index as usize,
                i+ 1,
                DEPTH
            ).unwrap();

        println!("On {}-th validator hash..", i);

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

    let poseidon_hash_tree_root = 
        compute_poseidon_hash_tree_root::<F,D>(2usize.pow(DEPTH as u32), validators_hashed);

    let hash = combine_two_hash_n_to_hash_no_pad::<F,D>(
        poseidon_validator_obj_vec[0].validator_poseidon_hash,
        poseidon_validator_obj_vec[1].validator_poseidon_hash,
    );
    println!("My Hash: {:?}\nProof Hash: {:?}\n", hash, poseidon_validator_obj_vec[2].validator_poseidon_proof[1]);

    let hash2 = combine_two_hash_n_to_hash_no_pad::<F,D>(
        poseidon_validator_obj_vec[2].validator_poseidon_hash,
        poseidon_validator_obj_vec[3].validator_poseidon_hash,
    );
    println!("My Hash: {:?}\nProof Hash: {:?}\n", hash2, poseidon_validator_obj_vec[0].validator_poseidon_proof[1]);

    let hash3 = combine_two_hash_n_to_hash_no_pad::<F,D>(
        hash,
        hash2,
    );
    println!("My Hash: {:?}\nProof Hash: {:?}\n", hash3, poseidon_hash_tree_root);

    
    

    // let hash = combine_two_hash_n_to_hash_no_pad::<F,D>(
    //     poseidon_validator_obj_vec[0].validator_poseidon_hash,
    //     poseidon_validator_obj_vec[0].validator_poseidon_proof[0],
    // );
    // println!("My Hash: {:?}\nProof Hash: {:?}\n", hash, poseidon_validator_obj_vec[3].validator_poseidon_proof[1]);

    // let hash2 = combine_two_hash_n_to_hash_no_pad::<F,D>(
    //     hash,
    //     poseidon_validator_obj_vec[0].validator_poseidon_proof[1],
    // );
    // println!("My Hash: {:?}\nProof Hash: {:?}\n", hash2, poseidon_hash_tree_root);

    // println!("\n\nAnother!\n\n");

    // let hash_final = combine_two_hash_n_to_hash_no_pad::<F,D>(
    //     poseidon_validator_obj_vec[3].validator_poseidon_proof[1],
    //     poseidon_validator_obj_vec[0].validator_poseidon_proof[1],
    // );
    // println!("Hash Final: {:?}\nProof Hash: {:?}\n\n", hash_final, poseidon_hash_tree_root);

    // prove_validator_membership(
    //     poseidon_validator_obj_vec[0].clone(),
    //     0, 
    //     poseidon_hash_tree_root,
    //     DEPTH
    // );

    // let poseidon_data = PoseidonData {
    //     validators: poseidon_validator_obj_vec,
    //     poseidon_root: poseidon_hash_tree_root
    // };

    // let mut data_map = HashMap::new();
    // data_map.insert("data", poseidon_data);


    // let json_poseidon_out = serde_json::to_string(&data_map).expect("Failed to serialize");
    // std::fs::write(file_path_attestations_out, json_poseidon_out).expect("Failed to write file");
}

pub fn compute_all_validators_tree() { //TODO: All validators from beacon state, not form all attestations

    const D: usize = 2;
    type F = GoldilocksField;

    //TODO: validator count is - 911215, hashed validators are - 911203, why?
    let file_path_attestations = 
                "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400_unique.json";
    let file_path_attestations_out = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400_poseidon_unique.json";

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
    
    for i in 0..num_validators {

        let (_leaf, proof) = 
            merkle_tree.generate_proof::<F, D>(
                validators_raw[i].validator_index as usize,
                DEPTH
            ).unwrap();

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

        // TODO: this

        // if not succesfull go from using index to using gindex
        
    }

    end_time = Instant::now();
    duration = end_time - start_time;

    println!("Proof Generation Took: {}", duration.as_millis());

    let poseidon_hash_tree_root = 
        compute_poseidon_hash_tree_root::<F,D>(2usize.pow(DEPTH as u32), validators_hashed);

    let poseidon_data = PoseidonData {
        validators: poseidon_validator_obj_vec,
        poseidon_root: poseidon_hash_tree_root
    };

    let mut data_map = HashMap::new();
    data_map.insert("data", poseidon_data);


    let json_poseidon_out = serde_json::to_string(&data_map).expect("Failed to serialize");
    std::fs::write(file_path_attestations_out, json_poseidon_out).expect("Failed to write file");

} 
