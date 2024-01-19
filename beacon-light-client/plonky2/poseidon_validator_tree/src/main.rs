use num_bigint::BigUint;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hashing::hash_n_to_hash_no_pad;
use poseidon_validator_tree::tree::{
    compute_validator_poseidon_hash, return_every_validator_hash, MerkleTree, Validator,
};
use poseidon_validator_tree::validator_type::{ValidatorData, ValidatorIndexData};
use serde_json::from_str;
use std::fs::File;
use std::io::Read;
pub mod tree;

fn hex_string_to_bool_array(hex_string: &str) -> Vec<bool> {
    let bytes = hex::decode(hex_string).unwrap_or_else(|_| {
        panic!("Failed to decode hex string");
    });

    let bool_array: Vec<bool> = bytes
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1 == 1))
        .collect();

    bool_array
}

fn vec_bool_to_array_bool<const N: usize>(vec_bool: Vec<bool>) -> Option<[bool; N]> {
    if vec_bool.len() == N {
        let array_bool: [bool; N] = {
            let mut array = [false; N];
            for (i, &value) in vec_bool.iter().enumerate() {
                array[i] = value;
            }
            array
        };
        Some(array_bool)
    } else {
        None
    }
}

// fn route_file_to_hashmap(fpath: &str) -> ValidatorIndexData {
//     let route_file_as_string = fs::read_to_string(fpath).expect("Unable to read file");
//     let data: ValidatorIndexData = serde_json::from_str(&route_file_as_string).unwrap();
//     return data;
// }

// pub fn routes_from_file(fpath: &str) -> ValidatorIndexData {
//     let route_file_as_map: ValidatorIndexData = route_file_to_hashmap(fpath);
//     return route_file_as_map;
// }

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;

    let validator = Validator {
        pubkey: [
            true, false, true, true, true, false, false, false, false, false, true, false, false,
            true, false, false, true, false, true, true, false, true, false, true, true, true,
            true, false, true, true, false, true, true, true, true, false, false, false, true,
            true, false, false, true, true, true, false, true, false, false, true, true, true,
            true, false, true, true, false, false, false, false, false, true, false, true, true,
            false, true, false, false, false, true, true, false, true, true, true, true, false,
            false, false, true, false, true, false, true, false, false, false, false, true, false,
            false, true, false, true, true, false, false, false, true, true, false, false, false,
            false, false, true, true, true, false, true, true, false, true, false, false, true,
            false, true, true, true, true, false, false, false, true, true, true, true, true, true,
            false, false, true, true, true, true, true, false, true, true, false, true, true, true,
            false, false, false, true, false, false, true, false, true, false, false, true, true,
            false, false, true, true, true, false, false, true, false, false, true, false, false,
            false, true, false, true, true, false, true, true, false, false, true, false, true,
            true, false, false, true, true, true, true, true, false, false, false, true, false,
            true, false, true, false, false, false, false, true, true, false, false, true, false,
            false, true, false, true, true, true, true, true, false, true, false, false, true,
            true, false, true, false, true, true, false, false, true, true, true, true, false,
            true, true, false, true, true, true, false, false, true, false, true, false, false,
            false, true, false, true, true, true, true, false, true, false, true, false, false,
            false, false, false, false, true, true, false, false, false, false, true, false, false,
            false, false, false, false, true, true, true, false, true, false, true, true, false,
            false, true, true, false, false, true, true, true, true, false, false, false, true,
            true, false, true, false, false, false, true, false, false, true, false, false, false,
            false, false, false, false, false, false, true, true, true, true, false, true, false,
            true, true, false, false, false, false, false, true, false, true, true, true, false,
            true, true, false, false, false, true, true, true, true, true, false, false, true,
            false, true, false, true, true, true, true, true, false, true, false, true, false,
            false, true, true, true, false, false, true, false, true, false, true, true, true,
            true, false, false, true, false,
        ],
        withdrawal_credentials: [
            false, false, false, false, false, false, false, false, false, true, false, true,
            false, false, true, false, true, true, true, true, true, true, true, false, true, true,
            false, true, false, false, true, false, false, true, true, true, true, false, true,
            true, true, false, true, false, true, true, false, true, true, false, true, true, true,
            true, false, true, true, false, true, true, false, true, true, false, false, true,
            true, true, true, true, false, true, false, true, false, true, true, true, false,
            false, true, false, true, false, true, true, false, true, false, true, true, false,
            true, false, true, false, false, true, false, true, true, false, false, true, false,
            false, true, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, true, false, false, true, false, true,
            false, true, false, true, true, true, true, false, true, false, false, false, false,
            false, false, true, true, false, true, false, false, false, true, true, false, true,
            true, true, true, false, true, true, true, false, true, true, false, true, true, true,
            true, false, false, false, false, false, false, false, false, false, true, true, false,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, true, false, true, false, true, false, true, false, true, true, false,
            true, true, true, true, true, false, true, true, true, true, true, false, true, true,
            true, false, true, false, true, false, true, false, true, false, true, false, false,
            true, false, false, false, true, true, false, false, true, true, true, true, false,
            false, true, true, false, false, true, true,
        ],
        effective_balance: BigUint::from(32000000000 as u64),
        slashed: false,
        activation_eligibility_epoch: BigUint::from(0 as u8),
        activation_epoch: BigUint::from(0 as u8),
        exit_epoch: BigUint::from(18446744073709551615 as u64),
        withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
    };

    let validator1 = Validator {
        pubkey: [
            true, false, true, true, true, false, false, false, false, false, true, false, false,
            true, false, false, true, false, true, true, false, true, false, true, true, true,
            true, false, true, true, false, true, true, true, true, false, false, false, true,
            true, false, false, true, true, true, false, true, false, false, true, true, true,
            true, false, true, true, false, false, false, false, false, true, false, true, true,
            false, true, false, false, false, true, true, false, true, true, true, true, false,
            false, false, true, false, true, false, true, false, false, false, false, true, false,
            false, true, false, true, true, false, false, false, true, true, false, false, false,
            false, false, true, true, true, false, true, true, false, true, false, false, true,
            false, true, true, true, true, false, false, false, true, true, true, true, true, true,
            false, false, true, true, true, true, true, false, true, true, false, true, true, true,
            false, false, false, true, false, false, true, false, true, false, false, true, true,
            false, false, true, true, true, false, false, true, false, false, true, false, false,
            false, true, false, true, true, false, true, true, false, false, true, false, true,
            true, false, false, true, true, true, true, true, false, false, false, true, false,
            true, false, true, false, false, false, false, true, true, false, false, true, false,
            false, true, false, true, true, true, true, true, false, true, false, false, true,
            true, false, true, false, true, true, false, false, true, true, true, true, false,
            true, true, false, true, true, true, false, false, true, false, true, false, false,
            false, true, false, true, true, true, true, false, true, false, true, false, false,
            false, false, false, false, true, true, false, false, false, false, true, false, false,
            false, false, false, false, true, true, true, false, true, false, true, true, false,
            false, true, true, false, false, true, true, true, true, false, false, false, true,
            true, false, true, false, false, false, true, false, false, true, false, false, false,
            false, false, false, false, false, false, true, true, true, true, false, true, false,
            true, true, false, false, false, false, false, true, false, true, true, true, false,
            true, true, false, false, false, true, true, true, true, true, false, false, true,
            false, true, false, true, true, true, true, true, false, true, false, true, false,
            false, true, true, true, false, false, true, false, true, false, true, true, true,
            true, false, false, true, false,
        ],
        withdrawal_credentials: [
            false, false, false, false, false, false, false, false, false, true, false, true,
            false, false, true, false, true, true, true, true, true, true, true, false, true, true,
            false, true, false, false, true, false, false, true, true, true, true, false, true,
            true, true, false, true, false, true, true, false, true, true, false, true, true, true,
            true, false, true, true, false, true, true, false, true, true, false, false, true,
            true, true, true, true, false, true, false, true, false, true, true, true, false,
            false, true, false, true, false, true, true, false, true, false, true, true, false,
            true, false, true, false, false, true, false, true, true, false, false, true, false,
            false, true, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, true, false, false, true, false, true,
            false, true, false, true, true, true, true, false, true, false, false, false, false,
            false, false, true, true, false, true, false, false, false, true, true, false, true,
            true, true, true, false, true, true, true, false, true, true, false, true, true, true,
            true, false, false, false, false, false, false, false, false, false, true, true, false,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, true, false, true, false, true, false, true, false, true, true, false,
            true, true, true, true, true, false, true, true, true, true, true, false, true, true,
            true, false, true, false, true, false, true, false, true, false, true, false, false,
            true, false, false, false, true, true, false, false, true, true, true, true, false,
            false, true, true, false, false, true, true,
        ],
        effective_balance: BigUint::from(32000000001 as u64),
        slashed: false,
        activation_eligibility_epoch: BigUint::from(0 as u8),
        activation_epoch: BigUint::from(0 as u8),
        exit_epoch: BigUint::from(18446744073709551615 as u64),
        withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
    };

    let validator2 = Validator {
        pubkey: [
            true, false, true, true, true, false, false, false, false, false, true, false, false,
            true, false, false, true, false, true, true, false, true, false, true, true, true,
            true, false, true, true, false, true, true, true, true, false, false, false, true,
            true, false, false, true, true, true, false, true, false, false, true, true, true,
            true, false, true, true, false, false, false, false, false, true, false, true, true,
            false, true, false, false, false, true, true, false, true, true, true, true, false,
            false, false, true, false, true, false, true, false, false, false, false, true, false,
            false, true, false, true, true, false, false, false, true, true, false, false, false,
            false, false, true, true, true, false, true, true, false, true, false, false, true,
            false, true, true, true, true, false, false, false, true, true, true, true, true, true,
            false, false, true, true, true, true, true, false, true, true, false, true, true, true,
            false, false, false, true, false, false, true, false, true, false, false, true, true,
            false, false, true, true, true, false, false, true, false, false, true, false, false,
            false, true, false, true, true, false, true, true, false, false, true, false, true,
            true, false, false, true, true, true, true, true, false, false, false, true, false,
            true, false, true, false, false, false, false, true, true, false, false, true, false,
            false, true, false, true, true, true, true, true, false, true, false, false, true,
            true, false, true, false, true, true, false, false, true, true, true, true, false,
            true, true, false, true, true, true, false, false, true, false, true, false, false,
            false, true, false, true, true, true, true, false, true, false, true, false, false,
            false, false, false, false, true, true, false, false, false, false, true, false, false,
            false, false, false, false, true, true, true, false, true, false, true, true, false,
            false, true, true, false, false, true, true, true, true, false, false, false, true,
            true, false, true, false, false, false, true, false, false, true, false, false, false,
            false, false, false, false, false, false, true, true, true, true, false, true, false,
            true, true, false, false, false, false, false, true, false, true, true, true, false,
            true, true, false, false, false, true, true, true, true, true, false, false, true,
            false, true, false, true, true, true, true, true, false, true, false, true, false,
            false, true, true, true, false, false, true, false, true, false, true, true, true,
            true, false, false, true, false,
        ],
        withdrawal_credentials: [
            false, false, false, false, false, false, false, false, false, true, false, true,
            false, false, true, false, true, true, true, true, true, true, true, false, true, true,
            false, true, false, false, true, false, false, true, true, true, true, false, true,
            true, true, false, true, false, true, true, false, true, true, false, true, true, true,
            true, false, true, true, false, true, true, false, true, true, false, false, true,
            true, true, true, true, false, true, false, true, false, true, true, true, false,
            false, true, false, true, false, true, true, false, true, false, true, true, false,
            true, false, true, false, false, true, false, true, true, false, false, true, false,
            false, true, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, true, false, false, true, false, true,
            false, true, false, true, true, true, true, false, true, false, false, false, false,
            false, false, true, true, false, true, false, false, false, true, true, false, true,
            true, true, true, false, true, true, true, false, true, true, false, true, true, true,
            true, false, false, false, false, false, false, false, false, false, true, true, false,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, true, false, true, false, true, false, true, false, true, true, false,
            true, true, true, true, true, false, true, true, true, true, true, false, true, true,
            true, false, true, false, true, false, true, false, true, false, true, false, false,
            true, false, false, false, true, true, false, false, true, true, true, true, false,
            false, true, true, false, false, true, true,
        ],
        effective_balance: BigUint::from(32000000002 as u64),
        slashed: false,
        activation_eligibility_epoch: BigUint::from(0 as u8),
        activation_epoch: BigUint::from(0 as u8),
        exit_epoch: BigUint::from(18446744073709551615 as u64),
        withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
    };

    let validator3 = Validator {
        pubkey: [
            true, false, true, true, true, false, false, false, false, false, true, false, false,
            true, false, false, true, false, true, true, false, true, false, true, true, true,
            true, false, true, true, false, true, true, true, true, false, false, false, true,
            true, false, false, true, true, true, false, true, false, false, true, true, true,
            true, false, true, true, false, false, false, false, false, true, false, true, true,
            false, true, false, false, false, true, true, false, true, true, true, true, false,
            false, false, true, false, true, false, true, false, false, false, false, true, false,
            false, true, false, true, true, false, false, false, true, true, false, false, false,
            false, false, true, true, true, false, true, true, false, true, false, false, true,
            false, true, true, true, true, false, false, false, true, true, true, true, true, true,
            false, false, true, true, true, true, true, false, true, true, false, true, true, true,
            false, false, false, true, false, false, true, false, true, false, false, true, true,
            false, false, true, true, true, false, false, true, false, false, true, false, false,
            false, true, false, true, true, false, true, true, false, false, true, false, true,
            true, false, false, true, true, true, true, true, false, false, false, true, false,
            true, false, true, false, false, false, false, true, true, false, false, true, false,
            false, true, false, true, true, true, true, true, false, true, false, false, true,
            true, false, true, false, true, true, false, false, true, true, true, true, false,
            true, true, false, true, true, true, false, false, true, false, true, false, false,
            false, true, false, true, true, true, true, false, true, false, true, false, false,
            false, false, false, false, true, true, false, false, false, false, true, false, false,
            false, false, false, false, true, true, true, false, true, false, true, true, false,
            false, true, true, false, false, true, true, true, true, false, false, false, true,
            true, false, true, false, false, false, true, false, false, true, false, false, false,
            false, false, false, false, false, false, true, true, true, true, false, true, false,
            true, true, false, false, false, false, false, true, false, true, true, true, false,
            true, true, false, false, false, true, true, true, true, true, false, false, true,
            false, true, false, true, true, true, true, true, false, true, false, true, false,
            false, true, true, true, false, false, true, false, true, false, true, true, true,
            true, false, false, true, false,
        ],
        withdrawal_credentials: [
            false, false, false, false, false, false, false, false, false, true, false, true,
            false, false, true, false, true, true, true, true, true, true, true, false, true, true,
            false, true, false, false, true, false, false, true, true, true, true, false, true,
            true, true, false, true, false, true, true, false, true, true, false, true, true, true,
            true, false, true, true, false, true, true, false, true, true, false, false, true,
            true, true, true, true, false, true, false, true, false, true, true, true, false,
            false, true, false, true, false, true, true, false, true, false, true, true, false,
            true, false, true, false, false, true, false, true, true, false, false, true, false,
            false, true, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, true, false, false, true, false, true,
            false, true, false, true, true, true, true, false, true, false, false, false, false,
            false, false, true, true, false, true, false, false, false, true, true, false, true,
            true, true, true, false, true, true, true, false, true, true, false, true, true, true,
            true, false, false, false, false, false, false, false, false, false, true, true, false,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, true, false, true, false, true, false, true, false, true, true, false,
            true, true, true, true, true, false, true, true, true, true, true, false, true, true,
            true, false, true, false, true, false, true, false, true, false, true, false, false,
            true, false, false, false, true, true, false, false, true, true, true, true, false,
            false, true, true, false, false, true, true,
        ],
        effective_balance: BigUint::from(32000000003 as u64),
        slashed: false,
        activation_eligibility_epoch: BigUint::from(0 as u8),
        activation_epoch: BigUint::from(0 as u8),
        exit_epoch: BigUint::from(18446744073709551615 as u64),
        withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
    };

    let validator4 = Validator {
        pubkey: [
            true, false, true, true, true, false, false, false, false, false, true, false, false,
            true, false, false, true, false, true, true, false, true, false, true, true, true,
            true, false, true, true, false, true, true, true, true, false, false, false, true,
            true, false, false, true, true, true, false, true, false, false, true, true, true,
            true, false, true, true, false, false, false, false, false, true, false, true, true,
            false, true, false, false, false, true, true, false, true, true, true, true, false,
            false, false, true, false, true, false, true, false, false, false, false, true, false,
            false, true, false, true, true, false, false, false, true, true, false, false, false,
            false, false, true, true, true, false, true, true, false, true, false, false, true,
            false, true, true, true, true, false, false, false, true, true, true, true, true, true,
            false, false, true, true, true, true, true, false, true, true, false, true, true, true,
            false, false, false, true, false, false, true, false, true, false, false, true, true,
            false, false, true, true, true, false, false, true, false, false, true, false, false,
            false, true, false, true, true, false, true, true, false, false, true, false, true,
            true, false, false, true, true, true, true, true, false, false, false, true, false,
            true, false, true, false, false, false, false, true, true, false, false, true, false,
            false, true, false, true, true, true, true, true, false, true, false, false, true,
            true, false, true, false, true, true, false, false, true, true, true, true, false,
            true, true, false, true, true, true, false, false, true, false, true, false, false,
            false, true, false, true, true, true, true, false, true, false, true, false, false,
            false, false, false, false, true, true, false, false, false, false, true, false, false,
            false, false, false, false, true, true, true, false, true, false, true, true, false,
            false, true, true, false, false, true, true, true, true, false, false, false, true,
            true, false, true, false, false, false, true, false, false, true, false, false, false,
            false, false, false, false, false, false, true, true, true, true, false, true, false,
            true, true, false, false, false, false, false, true, false, true, true, true, false,
            true, true, false, false, false, true, true, true, true, true, false, false, true,
            false, true, false, true, true, true, true, true, false, true, false, true, false,
            false, true, true, true, false, false, true, false, true, false, true, true, true,
            true, false, false, true, false,
        ],
        withdrawal_credentials: [
            false, false, false, false, false, false, false, false, false, true, false, true,
            false, false, true, false, true, true, true, true, true, true, true, false, true, true,
            false, true, false, false, true, false, false, true, true, true, true, false, true,
            true, true, false, true, false, true, true, false, true, true, false, true, true, true,
            true, false, true, true, false, true, true, false, true, true, false, false, true,
            true, true, true, true, false, true, false, true, false, true, true, true, false,
            false, true, false, true, false, true, true, false, true, false, true, true, false,
            true, false, true, false, false, true, false, true, true, false, false, true, false,
            false, true, false, true, false, false, true, true, false, true, false, true, true,
            false, false, false, true, false, false, false, true, false, false, true, false, true,
            false, true, false, true, true, true, true, false, true, false, false, false, false,
            false, false, true, true, false, true, false, false, false, true, true, false, true,
            true, true, true, false, true, true, true, false, true, true, false, true, true, true,
            true, false, false, false, false, false, false, false, false, false, true, true, false,
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, false, true, false, true, false, true, false, true, false, true, true, false,
            true, true, true, true, true, false, true, true, true, true, true, false, true, true,
            true, false, true, false, true, false, true, false, true, false, true, false, false,
            true, false, false, false, true, true, false, false, true, true, true, true, false,
            false, true, true, false, false, true, true,
        ],
        effective_balance: BigUint::from(32000000004 as u64),
        slashed: false,
        activation_eligibility_epoch: BigUint::from(0 as u8),
        activation_epoch: BigUint::from(0 as u8),
        exit_epoch: BigUint::from(18446744073709551615 as u64),
        withdrawable_epoch: BigUint::from(18446744073709551615 as u64),
    };

    let file_path = "./src/db.1706086784415.redis";

    let mut file_content = String::new();
    File::open(file_path)
        .expect("Failed to open file")
        .read_to_string(&mut file_content)
        .expect("Failed to read file");

    // let validator_index_data: ValidatorIndexData = routes_from_file(file_path);

    // let vectors: Vec<ValidatorData> = validator_index_data

    // let validators_from_json: Vec<ValidatorDataIndex> =
    //     from_str(&file_content).expect("Failed to parse JSON");

    // let mut validators: Vec<Validator> = Vec::with_capacity(validators_from_json.iter().count());
    // for validator in validators_from_json.iter() {
    //     validators.push(Validator {
    //         pubkey: vec_bool_to_array_bool(validator.pubkey.clone()).unwrap(),
    //         withdrawal_credentials: vec_bool_to_array_bool(
    //             validator.withdrawal_credentials.clone(),
    //         )
    //         .unwrap(),
    //         effective_balance: BigUint::from(validator.effective_balance),
    //         slashed: validator.slashed,
    //         activation_eligibility_epoch: BigUint::from(validator.activation_eligibility_epoch),
    //         activation_epoch: BigUint::from(validator.activation_epoch),
    //         exit_epoch: BigUint::from(validator.exit_epoch),
    //         withdrawable_epoch: BigUint::from(validator.withdrawable_epoch),
    //     });
    // }

    let _even_validators = vec![
        validator.clone(),
        validator1.clone(),
        validator2.clone(),
        validator3.clone(),
    ];

    let every_validator_hashed = return_every_validator_hash::<F, D>(_even_validators);

    let merkle_tree = MerkleTree::new::<F, D>(&every_validator_hashed, 2);
    // println!("merkle_tree is: {:?}", merkle_tree);
    if let Ok((merkle_node, merkle_node_path)) = merkle_tree.generate_proof::<F, D>(0, 2) {
        println!("merkle_node_path is: {:?}", merkle_node_path);
        println!("merkle_node is: {:?}", merkle_node);
    } else {
        print!("Error")
    }

    println!("||||||||||||||||||||||||||||||||||||||||");

    println!(
        "validator hash is: {:?}",
        compute_validator_poseidon_hash::<F, D>(validator)
    );
    println!(
        "validator1 hash is: {:?}",
        compute_validator_poseidon_hash::<F, D>(validator1)
    );
    println!(
        "validator2 hash is: {:?}",
        compute_validator_poseidon_hash::<F, D>(validator2)
    );
    println!(
        "validator3 hash is: {:?}",
        compute_validator_poseidon_hash::<F, D>(validator3)
    );
    println!(
        "validator4 hash is: {:?}",
        compute_validator_poseidon_hash::<F, D>(validator4)
    );
}
