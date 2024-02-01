use std::time::{Instant, Duration};
use num_bigint::BigUint;
use plonky2::{field::{goldilocks_field::GoldilocksField, types::PrimeField64}};
use poseidon_validator_tree::{
    objects::{Validator, ValidatorPoseidonDataOutput},
    parse_validators::{read_validator_data, binary_to_hex}, 
    tree::{compute_validator_poseidon_hash, compute_validators_merkle_proof, return_every_validator_hash, MerkleTree}
};

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

    let file_path_attestations = 
                "/home/stefan/code/repos/metacraft-labs/DendrETH/casper-finality-proofs/data/full_validator_set.json";

    let  validators_raw: Vec<Validator> = read_validator_data(file_path_attestations);

    let mut poseidon_validator_obj_vec = vec![];
    let mut validators_hashed = vec![];

    const DEPTH: usize = 40;

    let mut start_time = Instant::now();

    for i in 0..validators_raw.len() {
        let cur_validator_hash = compute_validator_poseidon_hash::<F,D>(validators_raw[i].clone());
        validators_hashed.push(cur_validator_hash);

        // println!("{:?}\n", validators_raw[i].activation_eligibility_epoch.to_u64_digits());

        // println!("{:?}",validators_raw[i]);

        poseidon_validator_obj_vec.push(
            ValidatorPoseidonDataOutput {
                    activation_eligibility_epoch: biguint_to_u64_unsafe(validators_raw[i].activation_eligibility_epoch.clone()),
                    activation_epoch: biguint_to_u64_unsafe(validators_raw[i].activation_epoch.clone()),
                    effective_balance: biguint_to_u64_unsafe(validators_raw[i].effective_balance.clone()),
                    exit_epoch: biguint_to_u64_unsafe(validators_raw[i].exit_epoch.clone()),
                    pubkey: binary_to_hex(validators_raw[i].pubkey.as_slice()),
                    slashed: validators_raw[i].slashed,
                    withdrawable_epoch: biguint_to_u64_unsafe(validators_raw[i].withdrawable_epoch.clone()),
                    withdrawal_credentials: binary_to_hex(validators_raw[i].withdrawal_credentials.as_slice()),
                    poseidon_proof: cur_validator_hash.elements
                }
            )

    }

    let mut end_time = Instant::now();
    let mut duration = end_time - start_time;
    println!("Casting Validators took: {}", duration.as_millis());
    start_time = Instant::now();

    let merkle_tree = MerkleTree::new::<F, D>(&validators_hashed, DEPTH);

    end_time = Instant::now();
    duration = end_time - start_time;
    println!("HashTreeRoot Generation Took: {}", duration.as_millis());
    start_time = Instant::now();

    let mut proofs = vec![];

    for i in 0..poseidon_validator_obj_vec.len() {
        proofs.push(merkle_tree.generate_proof::<F, D>(i, DEPTH).unwrap());

        if i%1000 == 0 {
            println!("{}",i);
        }
    }
    println!("{:?}", proofs.len());

    end_time = Instant::now();
    duration = end_time - start_time;
    println!("Proof Generation Took: {}", duration.as_millis());

}
