use crate::serialization::targets_serialization::{ReadTargets, WriteTargets};
use crate::{
    utils::{biguint::BigUintTarget, hashing::validator_hash_tree_root_poseidon::ValidatorTarget},
    withdrawal_credentials_balance_aggregator::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::SerializableCircuit;
use itertools::Itertools;
use plonky2::util::serialization::{Buffer, IoResult, Read, Write};

// serialize
// deserialize
// serialize_targets
// deserialize_targets

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> SerializableCircuit
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn serialize(targets: &Self::Targets) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.extend(BigUintTarget::write_targets(&targets.range_total_value)?);
        data.write_target_bool_vec(&targets.range_balances_root)?;
        data.write_target_hash(&targets.range_validator_commitment)?;

        for validator in &targets.validators {
            data.extend(ValidatorTarget::write_targets(validator)?);
        }

        data.write_target_bool_vec(&targets.non_zero_validator_leaves_mask)?;

        for balance in &targets.balances {
            data.write_target_bool_vec(balance)?;
        }

        for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
            data.write_target_bool_vec(&targets.withdrawal_credentials[i])?;
        }

        data.extend(BigUintTarget::write_targets(&targets.current_epoch)?);

        data.write_target(targets.number_of_non_activated_validators)?;
        data.write_target(targets.number_of_active_validators)?;
        data.write_target(targets.number_of_exited_validators)?;

        Ok(data)
    }

    fn deserialize(data: &mut Buffer) -> IoResult<Self::Targets> {
        Ok(Self::Targets {
            range_total_value: BigUintTarget::read_targets(data)?,
            range_balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            range_validator_commitment: data.read_target_hash()?,
            validators: [(); VALIDATORS_COUNT]
                .map(|_| ValidatorTarget::read_targets(data).unwrap()),
            non_zero_validator_leaves_mask: data.read_target_bool_vec()?.try_into().unwrap(),
            balances: [0; VALIDATORS_COUNT / 4]
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap()),
            withdrawal_credentials: (0..WITHDRAWAL_CREDENTIALS_COUNT)
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                .collect_vec()
                .try_into()
                .unwrap(),
            current_epoch: BigUintTarget::read_targets(data)?,
            number_of_non_activated_validators: data.read_target()?,
            number_of_active_validators: data.read_target()?,
            number_of_exited_validators: data.read_target()?,
        })
    }
}
