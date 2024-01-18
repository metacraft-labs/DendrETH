use num_bigint::BigUint;
use plonky2::field::goldilocks_field::GoldilocksField;
use tree::compute_validator_poseidon_hash_tree_root;

use crate::tree::{MerkleTree, Validator};
pub mod tree;

fn main() {
    const D: usize = 2;
    type F = GoldilocksField;

    // TODO: parse json
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

    let even_validators = vec![
        validator.clone(),
        validator1.clone(),
        validator2.clone(),
        validator3.clone(),
    ];
    let odd_validators = vec![
        validator.clone(),
        validator1.clone(),
        validator2.clone(),
        validator3.clone(),
        validator4.clone(),
    ];

    let merkle_tree = MerkleTree::initialize::<F, D>(odd_validators);
    // println!("asd {:?}", merkle_tree);
    let proof_that_it_exists = merkle_tree.generate_validator_proof::<F, D>(validator4.clone());
    println!("asd {:?}", Some(proof_that_it_exists));

    // compute_validator_poseidon_hash_tree_root::<F, D>(validator);
}
