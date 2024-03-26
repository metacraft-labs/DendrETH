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
    biguint::BigUintTarget,
    utils::{ETH_SHA256_BIT_SIZE, POSEIDON_HASH_SIZE},
    validator_balance_circuit_accumulator::{
        validator_balance_accumulator_verification, ValidatorBalanceVerificationTargetsAccumulator,
    },
};

pub const RANGE_TOTAL_VALUE_PUB_INDEX: usize = 0; // size 2
pub const RANGE_START_PUB_INDEX: usize = 2; // size 1
pub const RANGE_END_PUB_INDEX: usize = 3; // size 1
pub const RANGE_DEPOSIT_COUNT: usize = 4; // size 1
pub const BALANCES_ROOT_PUB_INDEX: usize = 5; // size 256
pub const RANGE_VALIDATOR_ACCUMULATOR_PUB_INDEX: usize = 261; // size 4
pub const VALIDATORS_COMMITMENT_PUB_INDEX: usize = 265; // size 4
pub const CURRENT_ETH1_DEPOSIT_PUB_INDEX: usize = 269; // size 2
pub const CURRENT_EPOCH_PUB_INDEX: usize = 271; // size 2
pub const NUMBER_OF_NON_ACTIVATED_VALIDATORS_INDEX: usize = 273; // size 1
pub const NUMBER_OF_ACTIVE_VALIDATORS_INDEX: usize = 274; // size 1
pub const NUMBER_OF_EXITED_VALIDATORS_INDEX: usize = 275; // size 1

pub type ValidatorBalanceAccumulatorProof =
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>;

pub trait ValidatorBalanceAccumulatorProofExt {
    fn get_range_total_value(&self) -> BigUint;

    fn get_start_pub_index(&self) -> u64;

    fn get_end_pub_index(&self) -> u64;

    fn range_deposit_count(&self) -> u64;

    fn get_range_balances_root(&self) -> [u64; ETH_SHA256_BIT_SIZE];

    fn get_range_validator_accumulator_index(&self) -> [String; POSEIDON_HASH_SIZE];

    fn get_validator_commitment(&self) -> [String; POSEIDON_HASH_SIZE];

    fn get_current_eth1_deposit_index(&self) -> BigUint;

    fn get_current_epoch(&self) -> BigUint;

    fn get_number_of_non_activated_validators(&self) -> u64;

    fn get_number_of_active_validators(&self) -> u64;

    fn get_number_of_exited_validators(&self) -> u64;
}

impl ValidatorBalanceAccumulatorProofExt for ValidatorBalanceAccumulatorProof {
    fn get_range_total_value(&self) -> BigUint {
        BigUint::new(
            self.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
                .iter()
                .map(|x| (x.0 % GoldilocksField::ORDER) as u32)
                .collect(),
        )
    }

    fn get_current_epoch(&self) -> BigUint {
        BigUint::new(
            self.public_inputs[CURRENT_EPOCH_PUB_INDEX..CURRENT_EPOCH_PUB_INDEX + 2]
                .iter()
                .map(|x| (x.0 % GoldilocksField::ORDER) as u32)
                .collect(),
        )
    }

    fn get_start_pub_index(&self) -> u64 {
        self.public_inputs[RANGE_START_PUB_INDEX].0 % GoldilocksField::ORDER
    }

    fn get_end_pub_index(&self) -> u64 {
        self.public_inputs[RANGE_END_PUB_INDEX].0 % GoldilocksField::ORDER
    }

    fn range_deposit_count(&self) -> u64 {
        self.public_inputs[RANGE_DEPOSIT_COUNT].0 % GoldilocksField::ORDER
    }

    fn get_range_balances_root(&self) -> [u64; ETH_SHA256_BIT_SIZE] {
        self.public_inputs[BALANCES_ROOT_PUB_INDEX..BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE]
            .iter()
            .map(|x| (x.0 % GoldilocksField::ORDER))
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_range_validator_accumulator_index(&self) -> [String; POSEIDON_HASH_SIZE] {
        self.public_inputs[RANGE_VALIDATOR_ACCUMULATOR_PUB_INDEX
            ..RANGE_VALIDATOR_ACCUMULATOR_PUB_INDEX + POSEIDON_HASH_SIZE]
            .iter()
            .map(|x| (x.0 % GoldilocksField::ORDER).to_string())
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_validator_commitment(&self) -> [String; POSEIDON_HASH_SIZE] {
        self.public_inputs
            [VALIDATORS_COMMITMENT_PUB_INDEX..VALIDATORS_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE]
            .iter()
            .map(|x| (x.0 % GoldilocksField::ORDER).to_string())
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_current_eth1_deposit_index(&self) -> BigUint {
        BigUint::new(
            self.public_inputs[CURRENT_ETH1_DEPOSIT_PUB_INDEX..CURRENT_ETH1_DEPOSIT_PUB_INDEX + 2]
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

pub type ValidatorBalanceProofAccumulatorTargets = ProofWithPublicInputsTarget<2>;

pub trait ValidatorBalanceProofAccumulatorTargetsExt {
    fn get_range_total_value(&self) -> BigUintTarget;

    fn get_range_start(&self) -> Target;

    fn get_range_end(&self) -> Target;

    fn get_range_deposit_count(&self) -> Target;

    fn get_balances_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE];

    fn get_range_validator_accumulator(&self) -> HashOutTarget;

    fn get_validator_commitment(&self) -> HashOutTarget;

    fn get_current_eth1_deposit_index(&self) -> BigUintTarget;

    fn get_current_epoch(&self) -> BigUintTarget;

    fn get_number_of_non_activated_validators(&self) -> Target;

    fn get_number_of_active_validators(&self) -> Target;

    fn get_number_of_exited_validators(&self) -> Target;
}

impl ValidatorBalanceProofAccumulatorTargetsExt for ValidatorBalanceProofAccumulatorTargets {
    fn get_range_total_value(&self) -> BigUintTarget {
        BigUintTarget {
            limbs: self.public_inputs[RANGE_TOTAL_VALUE_PUB_INDEX..RANGE_TOTAL_VALUE_PUB_INDEX + 2]
                .iter()
                .cloned()
                .map(|x| U32Target(x))
                .collect_vec(),
        }
    }

    fn get_balances_root(&self) -> [BoolTarget; ETH_SHA256_BIT_SIZE] {
        self.public_inputs[BALANCES_ROOT_PUB_INDEX..BALANCES_ROOT_PUB_INDEX + ETH_SHA256_BIT_SIZE]
            .iter()
            .cloned()
            .map(|x| BoolTarget::new_unsafe(x))
            .collect_vec()
            .try_into()
            .unwrap()
    }

    fn get_validator_commitment(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.public_inputs[VALIDATORS_COMMITMENT_PUB_INDEX
                ..VALIDATORS_COMMITMENT_PUB_INDEX + POSEIDON_HASH_SIZE]
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

    fn get_range_start(&self) -> Target {
        self.public_inputs[RANGE_START_PUB_INDEX]
    }

    fn get_range_end(&self) -> Target {
        self.public_inputs[RANGE_END_PUB_INDEX]
    }

    fn get_range_deposit_count(&self) -> Target {
        self.public_inputs[RANGE_DEPOSIT_COUNT]
    }

    fn get_range_validator_accumulator(&self) -> HashOutTarget {
        HashOutTarget {
            elements: self.public_inputs[RANGE_VALIDATOR_ACCUMULATOR_PUB_INDEX
                ..RANGE_VALIDATOR_ACCUMULATOR_PUB_INDEX + POSEIDON_HASH_SIZE]
                .try_into()
                .unwrap(),
        }
    }

    fn get_current_eth1_deposit_index(&self) -> BigUintTarget {
        BigUintTarget {
            limbs: self.public_inputs
                [CURRENT_ETH1_DEPOSIT_PUB_INDEX..CURRENT_ETH1_DEPOSIT_PUB_INDEX + 2]
                .iter()
                .cloned()
                .map(|x| U32Target(x))
                .collect_vec(),
        }
    }
}

pub fn build_validator_balance_accumulator_circuit(
    validators_len: usize,
) -> (
    ValidatorBalanceVerificationTargetsAccumulator,
    plonky2::plonk::circuit_data::CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());
    let targets = validator_balance_accumulator_verification(&mut builder, validators_len);

    let circuit_data = builder.build::<C>();
    (targets, circuit_data)
}

pub fn set_public_variables(
    builder: &mut CircuitBuilder<plonky2::field::goldilocks_field::GoldilocksField, 2>,
    range_total_value: &BigUintTarget,
    range_start: Target,
    range_end: Target,
    range_deposit_count: Target,
    balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    range_validator_accumulator: HashOutTarget,
    validator_commitment: HashOutTarget,
    current_eth1_deposit_index: &BigUintTarget,
    current_epoch: &BigUintTarget,
    number_of_non_activated_validators: Target,
    number_of_active_validators: Target,
    number_of_exited_validators: Target,
) {
    builder.register_public_inputs(&range_total_value.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_input(range_start);

    builder.register_public_input(range_end);

    builder.register_public_input(range_deposit_count);

    builder.register_public_inputs(&balances_root.map(|x| x.target));

    builder.register_public_inputs(&range_validator_accumulator.elements);

    builder.register_public_inputs(&validator_commitment.elements);

    builder.register_public_inputs(
        &current_eth1_deposit_index
            .limbs
            .iter()
            .map(|x| x.0)
            .collect_vec(),
    );

    builder.register_public_inputs(&current_epoch.limbs.iter().map(|x| x.0).collect_vec());

    builder.register_public_input(number_of_non_activated_validators);

    builder.register_public_input(number_of_active_validators);

    builder.register_public_input(number_of_exited_validators);
}
