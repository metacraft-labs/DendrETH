use num_bigint::BigUint;
use plonky2::field::goldilocks_field::GoldilocksField;
use tree::compute_validator_poseidon_hash_tree_root;

use crate::tree::Validator;
pub mod tree;

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

    compute_validator_poseidon_hash_tree_root::<F, D>(validator);
}
