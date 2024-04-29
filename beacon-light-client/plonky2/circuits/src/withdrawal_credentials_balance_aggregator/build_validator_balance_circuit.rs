// TODO: get rid of this file
use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field64},
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        config::PoseidonGoldilocksConfig,
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::utils::{
    biguint::BigUintTarget,
    utils::{ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE},
};

pub const RANGE_TOTAL_VALUE_PUB_INDEX: usize = 0;
pub const RANGE_BALANCES_ROOT_PUB_INDEX: usize = 2;
pub const WITHDRAWAL_CREDENTIALS_PUB_INDEX: usize = 258;
pub const RANGE_VALIDATOR_COMMITMENT_PUB_INDEX: usize = 514;
pub const CURRENT_EPOCH_PUB_INDEX: usize = 518;
pub const NUMBER_OF_NON_ACTIVATED_VALIDATORS_INDEX: usize = 520;
pub const NUMBER_OF_ACTIVE_VALIDATORS_INDEX: usize = 521;
pub const NUMBER_OF_EXITED_VALIDATORS_INDEX: usize = 522;

pub type ValidatorBalanceProof<const N: usize> =
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>;

pub trait ValidatorBalanceProofExt<const N: usize> {
    fn get_range_total_value(&self) -> BigUint;

    fn get_range_balances_root(&self) -> [u64; ETH_SHA256_BIT_SIZE];

    fn get_withdrawal_credentials(&self) -> [[u64; ETH_SHA256_BIT_SIZE]; N];

    fn get_range_validator_commitment(&self) -> [String; POSEIDON_HASH_SIZE];

    fn get_current_epoch(&self) -> BigUint;

    fn get_number_of_non_activated_validators(&self) -> u64;

    fn get_number_of_active_validators(&self) -> u64;

    fn get_number_of_exited_validators(&self) -> u64;
}

impl<const N: usize> ValidatorBalanceProofExt<N> for ValidatorBalanceProof<N> {
    fn get_range_total_value(&self) -> BigUint {
        BigUint::new(
            self.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
                .iter()
                .map(|x| (x.0 % GoldilocksField::ORDER) as u32)
                .collect(),
        )
    }

    fn get_range_balances_root(&self) -> [u64; ETH_SHA256_BIT_SIZE] {
        self.public_inputs[RANGE_BALANCES_ROOT_PUB_INDEX..RANGE_BALANCES_ROOT_PUB_INDEX + 256]
            .iter()
            .map(|x| (x.0 % GoldilocksField::ORDER) as u64)
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_withdrawal_credentials(&self) -> [[u64; ETH_SHA256_BIT_SIZE]; N] {
        (0..N)
            .map(|i| {
                self.public_inputs[WITHDRAWAL_CREDENTIALS_PUB_INDEX + i * ETH_SHA256_BIT_SIZE
                    ..WITHDRAWAL_CREDENTIALS_PUB_INDEX + (i + 1) * ETH_SHA256_BIT_SIZE]
                    .iter()
                    .map(|x| (x.0 % GoldilocksField::ORDER) as u64)
                    .collect_vec()
                    .try_into()
                    .unwrap()
            })
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_range_validator_commitment(&self) -> [String; POSEIDON_HASH_SIZE] {
        self.public_inputs[RANGE_VALIDATOR_COMMITMENT_PUB_INDEX
            ..RANGE_VALIDATOR_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE]
            .iter()
            .map(|x| ((x.0 % GoldilocksField::ORDER) as u64).to_string())
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_current_epoch(&self) -> BigUint {
        BigUint::new(
            self.public_inputs[CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2]
                .iter()
                .map(|x| (x.0 % GoldilocksField::ORDER) as u32)
                .collect(),
        )
    }

    fn get_number_of_non_activated_validators(&self) -> u64 {
        self.public_inputs[NUMBER_OF_NON_ACTIVATED_VALIDATORS_INDEX].0 % GoldilocksField::ORDER
    }

    fn get_number_of_active_validators(&self) -> u64 {
        self.public_inputs[NUMBER_OF_ACTIVE_VALIDATORS_INDEX].0 % GoldilocksField::ORDER
    }

    fn get_number_of_exited_validators(&self) -> u64 {
        self.public_inputs[NUMBER_OF_EXITED_VALIDATORS_INDEX].0 % GoldilocksField::ORDER
    }
}

pub type ValidatorBalanceProofTargets<const N: usize> = ProofWithPublicInputsTarget<2>;

pub trait ValidatorBalanceProofTargetsExt<const N: usize> {
    fn get_range_total_value(&self) -> BigUintTarget;

    fn get_range_balances_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE];

    fn get_withdrawal_credentials(&self) -> [[BoolTarget; ETH_SHA256_BIT_SIZE]; N];

    fn get_range_validator_commitment(&self) -> HashOutTarget;

    fn get_current_epoch(&self) -> BigUintTarget;

    fn get_number_of_non_activated_validators(&self) -> Target;

    fn get_number_of_active_validators(&self) -> Target;

    fn get_number_of_exited_validators(&self) -> Target;
}

impl<const N: usize> ValidatorBalanceProofTargetsExt<N> for ValidatorBalanceProofTargets<N> {
    fn get_range_total_value(&self) -> BigUintTarget {
        BigUintTarget {
            limbs: self.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
                .iter()
                .cloned()
                .map(|x| U32Target(x))
                .collect_vec(),
        }
    }

    fn get_range_balances_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
        self.public_inputs
            [RANGE_BALANCES_ROOT_PUB_INDEX..RANGE_BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE]
            .iter()
            .cloned()
            .map(|x| BoolTarget::new_unsafe(x))
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_withdrawal_credentials(&self) -> [[BoolTarget; ETH_SHA256_BIT_SIZE]; N] {
        let mut r = [[BoolTarget::default(); ETH_SHA256_BIT_SIZE]; N];

        for i in 0..N {
            r[i] = self.public_inputs[WITHDRAWAL_CREDENTIALS_PUB_INDEX
                ..WITHDRAWAL_CREDENTIALS_PUB_INDEX + ETH_SHA256_BIT_SIZE]
                .iter()
                .cloned()
                .map(|x| BoolTarget::new_unsafe(x))
                .collect_vec()
                .try_into()
                .unwrap()
        }

        r
    }

    fn get_range_validator_commitment(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.public_inputs[RANGE_VALIDATOR_COMMITMENT_PUB_INDEX
                ..RANGE_VALIDATOR_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE]
                .try_into()
                .unwrap(),
        }
    }

    fn get_current_epoch(&self) -> BigUintTarget {
        BigUintTarget {
            limbs: self.public_inputs[CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2]
                .iter()
                .cloned()
                .map(|x| U32Target(x))
                .collect_vec(),
        }
    }

    fn get_number_of_non_activated_validators(&self) -> Target {
        self.public_inputs[NUMBER_OF_NON_ACTIVATED_VALIDATORS_INDEX]
    }

    fn get_number_of_active_validators(&self) -> Target {
        self.public_inputs[NUMBER_OF_ACTIVE_VALIDATORS_INDEX]
    }

    fn get_number_of_exited_validators(&self) -> Target {
        self.public_inputs[NUMBER_OF_EXITED_VALIDATORS_INDEX]
    }
}
