use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    is_active_validator::get_validator_status,
    is_valid_merkle_branch::is_valid_merkle_branch_sha256_result,
    is_valid_merkle_branch_poseidon::is_valid_merkle_branch_poseidon_result,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{create_bool_target_array, if_biguint, ssz_num_from_bits, ETH_SHA256_BIT_SIZE},
    validator_accumulator_commitment_mapper::get_validators_accumulator_leaves,
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidonHashTreeRootTargets,
        ValidatorPoseidonTargets,
    },
};

pub struct ValidatorBalanceVerificationTargetsAccumulator {
    pub range_total_value: BigUintTarget,
    pub range_start: Target,
    pub range_end: Target,
    pub range_deposit_count: Target,
    pub balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub balances: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub balances_proofs: Vec<Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>>,
    pub validator_commitment_root: HashOutTarget,
    pub accumulator_commitment_root: HashOutTarget,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub validator_accumulator_indexes: Vec<Target>,
    pub validator_deposit_indexes: Vec<BigUintTarget>,
    pub validator_indexes: Vec<Target>,
    pub validator_commitment_proofs: Vec<Vec<HashOutTarget>>,
    pub validator_accumulator_proofs: Vec<Vec<HashOutTarget>>,
    pub validator_is_not_zero: Vec<BoolTarget>,
    pub current_epoch: BigUintTarget,
    pub current_eth1_deposit_index: BigUintTarget,
    pub number_of_non_activated_validators: Target,
    pub number_of_active_validators: Target,
    pub number_of_exited_validators: Target,
}

impl ReadTargets for ValidatorBalanceVerificationTargetsAccumulator {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorBalanceVerificationTargetsAccumulator> {
        let validators_len = data.read_usize()?;

        Ok(ValidatorBalanceVerificationTargetsAccumulator {
            range_total_value: BigUintTarget::read_targets(data)?,
            range_start: data.read_target()?,
            range_end: data.read_target()?,
            range_deposit_count: data.read_target()?,
            balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            balances: (0..validators_len / 4)
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                .collect(),
            balances_proofs: (0..validators_len / 4)
                .map(|_| {
                    (0..24)
                        .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                        .collect_vec()
                })
                .collect_vec(),
            validator_commitment_root: data.read_target_hash()?,
            accumulator_commitment_root: data.read_target_hash()?,
            validators: (0..validators_len)
                .map(|_| ValidatorPoseidonTargets::read_targets(data).unwrap())
                .collect(),
            validator_accumulator_indexes: data.read_target_vec()?,
            validator_deposit_indexes: (0..validators_len)
                .map(|_| BigUintTarget::read_targets(data).unwrap())
                .collect_vec(),
            validator_indexes: data.read_target_vec()?,
            validator_commitment_proofs: (0..validators_len)
                .map(|_| {
                    (0..24)
                        .map(|_| data.read_target_hash().unwrap())
                        .collect_vec()
                })
                .collect_vec(),
            validator_accumulator_proofs: (0..validators_len)
                .map(|_| {
                    (0..24)
                        .map(|_| data.read_target_hash().unwrap())
                        .collect_vec()
                })
                .collect_vec(),

            validator_is_not_zero: data.read_target_bool_vec()?,
            current_epoch: BigUintTarget::read_targets(data)?,
            current_eth1_deposit_index: BigUintTarget::read_targets(data)?,
            number_of_non_activated_validators: data.read_target()?,
            number_of_active_validators: data.read_target()?,
            number_of_exited_validators: data.read_target()?,
        })
    }
}

impl WriteTargets for ValidatorBalanceVerificationTargetsAccumulator {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_usize(self.validators.len())?;
        data.extend(BigUintTarget::write_targets(&self.range_total_value)?);
        data.write_target(self.range_start)?;
        data.write_target(self.range_end)?;
        data.write_target(self.range_deposit_count)?;
        data.write_target_bool_vec(&self.balances_root)?;

        for balance in &self.balances {
            data.write_target_bool_vec(balance)?;
        }

        for balance_proof in &self.balances_proofs {
            for element in balance_proof {
                data.write_target_bool_vec(element)?;
            }
        }

        data.write_target_hash(&self.validator_commitment_root)?;
        data.write_target_hash(&self.accumulator_commitment_root)?;

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        data.write_target_vec(&self.validator_accumulator_indexes)?;

        for validator_deposit_index in &self.validator_deposit_indexes {
            data.extend(BigUintTarget::write_targets(validator_deposit_index)?);
        }

        data.write_target_vec(&self.validator_indexes)?;

        for validator_proof in &self.validator_commitment_proofs {
            for element in validator_proof {
                data.write_target_hash(element)?;
            }
        }

        for validator_proof in &self.validator_accumulator_proofs {
            for element in validator_proof {
                data.write_target_hash(element)?;
            }
        }

        data.write_target_bool_vec(&self.validator_is_not_zero)?;

        data.extend(BigUintTarget::write_targets(&self.current_epoch)?);

        data.extend(BigUintTarget::write_targets(
            &self.current_eth1_deposit_index,
        )?);

        data.write_target(self.number_of_non_activated_validators)?;
        data.write_target(self.number_of_active_validators)?;
        data.write_target(self.number_of_exited_validators)?;

        Ok(data)
    }
}

pub fn validator_balance_accumulator_verification<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_len: usize,
) -> ValidatorBalanceVerificationTargetsAccumulator {
    if !validators_len.is_power_of_two() {
        panic!("validators_len must be a power of two");
    }

    let balances_leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..validators_len)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let balances_root = create_bool_target_array(builder);

    let mut balances_proofs = Vec::new();

    let validator_indexes = (0..validators_len)
        .map(|_| builder.add_virtual_target())
        .collect_vec();

    let validator_is_not_zero = (0..validators_len)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect_vec();

    let current_eth1_deposit_index = builder.add_virtual_biguint_target(2);

    let validator_deposit_indexes = (0..validators_len)
        .map(|_| builder.add_virtual_biguint_target(2))
        .collect_vec();

    for i in 0..balances_leaves.len() {
        let is_valid_merkle_branch_balance = is_valid_merkle_branch_sha256_result(builder, 22);

        balances_proofs.push(is_valid_merkle_branch_balance.branch);

        // TODO: adjust
        builder.connect(is_valid_merkle_branch_balance.index, validator_indexes[i]);

        for j in 0..256 {
            builder.connect(
                is_valid_merkle_branch_balance.leaf[j].target,
                balances_leaves[i][j].target,
            );

            builder.connect(
                is_valid_merkle_branch_balance.root[j].target,
                balances_root[j].target,
            );
        }

        let is_part_of_the_beacon_chain =
            builder.cmp_biguint(&validator_deposit_indexes[i], &current_eth1_deposit_index);

        let should_be_checked = builder.and(validator_is_not_zero[i], is_part_of_the_beacon_chain);

        builder.connect(
            is_valid_merkle_branch_balance.is_valid.target,
            should_be_checked.target,
        );
    }

    let validators_leaves: Vec<ValidatorPoseidonHashTreeRootTargets> = (0..validators_len)
        .map(|_| hash_tree_root_validator_poseidon(builder))
        .collect();

    let validator_accumulator_leaves: Vec<HashOutTarget> = get_validators_accumulator_leaves(
        builder,
        &validators_leaves
            .iter()
            .map(|x| x.validator.pubkey)
            .collect(),
        &validator_deposit_indexes,
    );

    let accumulator_commitment_root = builder.add_virtual_hash();
    let validator_commitment_root = builder.add_virtual_hash();

    let validator_accumulator_indexes = (0..validators_len)
        .map(|_| builder.add_virtual_target())
        .collect_vec();

    let validator_accumulator_proofs = (0..validators_len)
        .map(|_| (0..24).map(|_| builder.add_virtual_hash()).collect_vec())
        .collect_vec();

    let validator_commitment_proofs = (0..validators_len)
        .map(|_| (0..24).map(|_| builder.add_virtual_hash()).collect_vec())
        .collect_vec();

    for i in 0..validators_leaves.len() {
        let is_valid_merkle_branch_accumulator =
            is_valid_merkle_branch_poseidon_result(builder, 24);
        let is_valid_merkle_branch_commitment = is_valid_merkle_branch_poseidon_result(builder, 24);

        builder.connect_hashes(
            is_valid_merkle_branch_accumulator.root,
            accumulator_commitment_root,
        );
        builder.connect_hashes(
            is_valid_merkle_branch_accumulator.leaf,
            validator_accumulator_leaves[i],
        );
        builder.connect(
            is_valid_merkle_branch_accumulator.index,
            validator_accumulator_indexes[i],
        );

        for j in 0..24 {
            builder.connect_hashes(
                is_valid_merkle_branch_accumulator.branch[j],
                validator_accumulator_proofs[i][j],
            )
        }

        builder.connect(
            is_valid_merkle_branch_accumulator.is_valid.target,
            validator_is_not_zero[i].target,
        );

        builder.connect_hashes(
            is_valid_merkle_branch_commitment.root,
            validator_commitment_root,
        );
        builder.connect_hashes(
            is_valid_merkle_branch_commitment.leaf,
            validators_leaves[i].hash_tree_root,
        );
        builder.connect(
            is_valid_merkle_branch_commitment.index,
            validator_indexes[i],
        );

        for j in 0..24 {
            builder.connect_hashes(
                is_valid_merkle_branch_commitment.branch[j],
                validator_accumulator_proofs[i][j],
            );
        }

        let is_part_of_the_beacon_chain =
            builder.cmp_biguint(&validator_deposit_indexes[i], &current_eth1_deposit_index);
        let should_be_checked = builder.and(validator_is_not_zero[i], is_part_of_the_beacon_chain);

        builder.connect(
            is_valid_merkle_branch_commitment.is_valid.target,
            should_be_checked.target,
        );
    }

    let current_epoch = builder.add_virtual_biguint_target(2);

    let mut number_of_non_activated_validators = builder.add_virtual_target();
    let mut number_of_active_validators = builder.add_virtual_target();
    let mut number_of_exited_validators = builder.add_virtual_target();

    let mut range_total_value = builder.zero_biguint();

    let range_start = builder.add_virtual_target();
    let range_end = builder.add_virtual_target();

    let mut range_deposit_count = builder.zero();

    for i in 0..validators_len {
        // TODO: abstraction is missing same code as validator balance circuit
        let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
            get_validator_status(
                builder,
                &validators_leaves[i].validator.activation_epoch,
                &current_epoch,
                &validators_leaves[i].validator.exit_epoch,
            );

        let balance = ssz_num_from_bits(
            builder,
            &balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 64)],
        );

        let zero = builder.zero_biguint();

        let is_part_of_the_beacon_chain =
            builder.cmp_biguint(&validator_deposit_indexes[i], &current_eth1_deposit_index);

        let will_be_counted = builder.and(is_valid_validator, is_part_of_the_beacon_chain);

        let current = if_biguint(builder, will_be_counted, &balance, &zero);

        range_total_value = builder.add_biguint(&range_total_value, &current);

        number_of_active_validators =
            builder.add(number_of_active_validators, will_be_counted.target);

        let will_be_counted = builder.and(is_part_of_the_beacon_chain, is_non_activated_validator);

        number_of_non_activated_validators =
            builder.add(number_of_non_activated_validators, will_be_counted.target);

        let will_be_counted = builder.and(is_part_of_the_beacon_chain, is_exited_validator);

        number_of_exited_validators =
            builder.add(number_of_exited_validators, will_be_counted.target);

        range_deposit_count = builder.add(range_deposit_count, validator_is_not_zero[i].target);

        range_total_value.limbs.pop();
    }

    ValidatorBalanceVerificationTargetsAccumulator {
        range_total_value,
        range_start,
        range_end,
        range_deposit_count,
        balances_root,
        balances_proofs,
        validator_commitment_root,
        accumulator_commitment_root,
        validators: validators_leaves
            .iter()
            .map(|x| x.validator.clone())
            .collect_vec(),
        validator_deposit_indexes,
        validator_indexes,
        validator_accumulator_indexes,
        validator_commitment_proofs,
        validator_accumulator_proofs,
        validator_is_not_zero,
        balances: balances_leaves,
        current_epoch,
        current_eth1_deposit_index,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
    }
}
