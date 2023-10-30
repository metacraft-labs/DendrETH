use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},
    hash_tree_root::hash_tree_root,
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    is_active_validator::get_validator_status,
    targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        bool_target_equal, create_bool_target_array, if_biguint, ssz_num_from_bits,
        ETH_SHA256_BIT_SIZE,
    },
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidonHashTreeRootTargets,
        ValidatorPoseidonTargets,
    },
};

pub struct ValidatorBalanceVerificationTargets {
    pub range_total_value: BigUintTarget,
    pub range_balances_root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub range_validator_commitment: HashOutTarget,
    pub validators: Vec<ValidatorPoseidonTargets>,
    pub validator_is_zero: Vec<BoolTarget>,
    pub balances: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub withdrawal_credentials: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub current_epoch: BigUintTarget,
    pub number_of_non_activated_validators: Target,
    pub number_of_active_validators: Target,
    pub number_of_exited_validators: Target,
}

impl ReadTargets for ValidatorBalanceVerificationTargets {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorBalanceVerificationTargets> {
        let validators_len = data.read_usize()?;

        Ok(ValidatorBalanceVerificationTargets {
            range_total_value: BigUintTarget::read_targets(data)?,
            range_balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            range_validator_commitment: data.read_target_hash()?,
            validators: (0..validators_len)
                .map(|_| ValidatorPoseidonTargets::read_targets(data).unwrap())
                .collect(),
            validator_is_zero: data.read_target_bool_vec()?,
            balances: (0..validators_len / 4)
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                .collect(),
            withdrawal_credentials: data.read_target_bool_vec()?.try_into().unwrap(),
            current_epoch: BigUintTarget::read_targets(data)?,
            number_of_non_activated_validators: data.read_target()?,
            number_of_active_validators: data.read_target()?,
            number_of_exited_validators: data.read_target()?,
        })
    }
}

impl WriteTargets for ValidatorBalanceVerificationTargets {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_usize(self.validators.len())?;
        data.extend(BigUintTarget::write_targets(&self.range_total_value)?);
        data.write_target_bool_vec(&self.range_balances_root)?;
        data.write_target_hash(&self.range_validator_commitment)?;

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        data.write_target_bool_vec(&self.validator_is_zero)?;

        for balance in &self.balances {
            data.write_target_bool_vec(balance)?;
        }

        data.write_target_bool_vec(&self.withdrawal_credentials)?;

        data.extend(BigUintTarget::write_targets(&self.current_epoch)?);

        data.write_target(self.number_of_non_activated_validators)?;
        data.write_target(self.number_of_active_validators)?;
        data.write_target(self.number_of_exited_validators)?;

        Ok(data)
    }
}

pub fn validator_balance_verification<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_len: usize,
) -> ValidatorBalanceVerificationTargets {
    if !validators_len.is_power_of_two() {
        panic!("validators_len must be a power of two");
    }

    let balances_len = validators_len / 4;

    let balances_leaves: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..balances_len)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let balances_hash_tree_root_targets = hash_tree_root(builder, balances_len);

    for i in 0..balances_len {
        for j in 0..ETH_SHA256_BIT_SIZE {
            builder.connect(
                balances_hash_tree_root_targets.leaves[i][j].target,
                balances_leaves[i][j].target,
            );
        }
    }

    let validators_leaves: Vec<ValidatorPoseidonHashTreeRootTargets> = (0..validators_len)
        .map(|_| hash_tree_root_validator_poseidon(builder))
        .collect();

    let hash_tree_root_poseidon_targets = hash_tree_root_poseidon(builder, validators_len);

    let validator_is_zero: Vec<BoolTarget> = (0..validators_len)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect();

    let zero_hash = builder.zero();

    for i in 0..validators_len {
        let mut elements = [zero_hash; 4];

        for (j, _) in validators_leaves[i]
            .hash_tree_root
            .elements
            .iter()
            .enumerate()
        {
            elements[j] = builder._if(
                validator_is_zero[i],
                zero_hash,
                validators_leaves[i].hash_tree_root.elements[j],
            );
        }

        builder.connect_hashes(
            hash_tree_root_poseidon_targets.leaves[i],
            HashOutTarget { elements },
        );
    }

    let withdrawal_credentials = create_bool_target_array(builder);

    let current_epoch = builder.add_virtual_biguint_target(2);

    let mut sum = builder.zero_biguint();

    let mut number_of_non_activated_validators = builder.zero();

    let mut number_of_active_validators = builder.zero();

    let mut number_of_exited_validators = builder.zero();

    for i in 0..validators_len {
        let is_equal = bool_target_equal(
            builder,
            &validators_leaves[i].validator.withdrawal_credentials,
            &withdrawal_credentials,
        );

        let balance = ssz_num_from_bits(
            builder,
            &balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 64)],
        );

        let zero = builder.zero_biguint();

        let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
            get_validator_status(
                builder,
                &validators_leaves[i].validator.activation_epoch,
                &current_epoch,
                &validators_leaves[i].validator.exit_epoch,
            );

        let will_be_counted = builder.and(is_equal, is_valid_validator);

        let current = if_biguint(builder, will_be_counted, &balance, &zero);

        sum = builder.add_biguint(&sum, &current);

        number_of_active_validators =
            builder.add(number_of_active_validators, will_be_counted.target);

        let will_be_counted = builder.and(is_equal, is_non_activated_validator);

        number_of_non_activated_validators =
            builder.add(number_of_non_activated_validators, will_be_counted.target);

        let will_be_counted = builder.and(is_equal, is_exited_validator);

        number_of_exited_validators =
            builder.add(number_of_exited_validators, will_be_counted.target);

        sum.limbs.pop();
    }

    ValidatorBalanceVerificationTargets {
        validator_is_zero: validator_is_zero,
        range_total_value: sum,
        range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
        range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
        validators: validators_leaves
            .iter()
            .map(|v| v.validator.clone())
            .collect(),
        balances: balances_leaves,
        withdrawal_credentials: withdrawal_credentials,
        current_epoch,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
    }
}
