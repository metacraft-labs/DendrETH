use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field64},
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::{
    biguint::BigUintTarget, types::ValidatorBalanceProofTargets, utils::{ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE}, validator_balance_circuit::{
        validator_balance_verification, ValidatorBalanceVerificationTargets,
    }
};

pub const RANGE_TOTAL_VALUE_PUB_INDEX: usize = 0;
pub const RANGE_BALANCES_ROOT_PUB_INDEX: usize = 2;
pub const WITHDRAWAL_CREDENTIALS_PUB_INDEX: usize = 258;
pub const RANGE_VALIDATOR_COMMITMENT_PUB_INDEX: usize = 514;
pub const CURRENT_EPOCH_PUB_INDEX: usize = 518;
pub const NUMBER_OF_NON_ACTIVATED_VALIDATORS_INDEX: usize = 520;
pub const NUMBER_OF_ACTIVE_VALIDATORS_INDEX: usize = 521;
pub const NUMBER_OF_EXITED_VALIDATORS_INDEX: usize = 522;
pub const NUMBER_OF_SLASHED_VALIDATORS_INDEX: usize = 523;

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

    fn get_number_of_slashed_validators(&self) -> u64;
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

    fn get_number_of_slashed_validators(&self) -> u64 {
        self.public_inputs[NUMBER_OF_SLASHED_VALIDATORS_INDEX].0 % GoldilocksField::ORDER
    }
}

pub trait ValidatorBalanceProofTargetsExt<const N: usize> {
    fn get_range_total_value(&self) -> BigUintTarget;

    fn get_range_balances_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE];

    fn get_withdrawal_credentials(&self) -> [[BoolTarget; ETH_SHA256_BIT_SIZE]; N];

    fn get_range_validator_commitment(&self) -> HashOutTarget;

    fn get_current_epoch(&self) -> BigUintTarget;

    fn get_number_of_non_activated_validators(&self) -> Target;

    fn get_number_of_active_validators(&self) -> Target;

    fn get_number_of_exited_validators(&self) -> Target;

    fn get_number_of_slashed_validators(&self) -> Target;
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

    fn get_number_of_slashed_validators(&self) -> Target {
        self.public_inputs[NUMBER_OF_SLASHED_VALIDATORS_INDEX]
    }
}

pub fn build_validator_balance_circuit<const N: usize>(
    validators_len: usize,
) -> (
    ValidatorBalanceVerificationTargets<N>,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let validator_balance_verification_targets =
        validator_balance_verification(&mut builder, validators_len);

    set_public_variables(
        &mut builder,
        &validator_balance_verification_targets.range_total_value,
        validator_balance_verification_targets.range_balances_root,
        &validator_balance_verification_targets.withdrawal_credentials,
        validator_balance_verification_targets.range_validator_commitment,
        &validator_balance_verification_targets.current_epoch,
        validator_balance_verification_targets.number_of_non_activated_validators,
        validator_balance_verification_targets.number_of_active_validators,
        validator_balance_verification_targets.number_of_exited_validators,
        validator_balance_verification_targets.number_of_slashed_validators,
    );

    let data = builder.build::<C>();

    (validator_balance_verification_targets, data)
}

pub fn set_public_variables<const N: usize>(
    builder: &mut CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2>,
    range_total_value: &BigUintTarget,
    range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    withdrawal_credentials: &[[BoolTarget; ETH_SHA256_BIT_SIZE]; N],
    range_validator_commitment: HashOutTarget,
    current_epoch: &BigUintTarget,
    number_of_non_activated_validators: Target,
    number_of_active_validators: Target,
    number_of_exited_validators: Target,
    number_of_slashed_validators: Target,
) {
    builder.register_public_inputs(&range_total_value.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_inputs(&range_balances_root.map(|x| x.target));

    for i in 0..N {
        builder.register_public_inputs(
            &withdrawal_credentials[i]
                .iter()
                .map(|x| x.target)
                .collect_vec(),
        );
    }

    builder.register_public_inputs(&range_validator_commitment.elements);

    builder.register_public_inputs(&current_epoch.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_input(number_of_non_activated_validators);

    builder.register_public_input(number_of_active_validators);

    builder.register_public_input(number_of_exited_validators);

    builder.register_public_input(number_of_slashed_validators);
}
