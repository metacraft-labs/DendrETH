use std::{collections::HashMap, num, time::Instant};
use casper_finality_proofs::utils::{poseidon_helpers::{binary_to_hex, ValidatorPoseidonDataOutput}, poseidon_merkle_tree::{compute_merkle_hash_tree, get_validator_proof, gindex_from_validator_index, parse_validator_data, zero_hashes, MAX_DEPTH}};
use num_bigint::BigUint;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::{hash_types::HashOut, poseidon::PoseidonPermutation}};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PoseidonData {
    validators: Vec<ValidatorPoseidonDataOutput>,
    poseidon_root: HashOut<GoldilocksField>,
}

fn biguint_to_u64_unsafe(x: BigUint) -> u64 {
    let result = x.to_u64_digits();
    if result.is_empty() {0} else {result[0]}
}

pub fn main() {

    let file_path_toy_data = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400.json";
    let file_path_toy_data_out = 
        "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/all_validators_234400_poseidon.json";

    let poseidon_data = compute_all_validators_tree(file_path_toy_data);

    let mut data_map = HashMap::new();
    data_map.insert("data", poseidon_data);

    let json_poseidon_out = serde_json::to_string(&data_map).expect("Failed to serialize");
    std::fs::write(file_path_toy_data_out, json_poseidon_out).expect("Failed to write file");

}

pub fn compute_all_validators_tree(file_path: &str) -> PoseidonData
{
    let (validators_raw,validators_hashed) = parse_validator_data(file_path);

    let mut poseidon_validator_obj_vec = Vec::with_capacity(validators_hashed.len());

    let mut start_time = Instant::now();
    
    let validator_map = compute_merkle_hash_tree(
        &validators_hashed, 
    );
    
    let mut duration = start_time.elapsed();
    println!("Building tree took: {}", duration.as_secs());
    
    let zero_hashes = zero_hashes();
    
    start_time = Instant::now();
    for i in 0..validators_hashed.len() {

        let validator_gindex = gindex_from_validator_index(i as u64, MAX_DEPTH as u32);
        let proof = get_validator_proof(validator_gindex, &validator_map, &zero_hashes);


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

    duration = start_time.elapsed();

    println!("Proof Generation Took: {}", duration.as_secs());

    let poseidon_data = PoseidonData {
        validators: poseidon_validator_obj_vec,
        poseidon_root: validator_map[&0]
    };

    poseidon_data

} 

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::compute_all_validators_tree;

    #[test]
    fn parse_toy_data() {

        let file_path_toy_data = 
            "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data.json";
        let file_path_toy_data_out = 
            "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/poseidon_toy_data_out.json";

        let poseidon_data = compute_all_validators_tree(file_path_toy_data);

        let mut data_map = HashMap::new();
        data_map.insert("data", poseidon_data);

        let json_poseidon_out = serde_json::to_string(&data_map).expect("Failed to serialize");
        std::fs::write(file_path_toy_data_out, json_poseidon_out).expect("Failed to write file");

    }
}
