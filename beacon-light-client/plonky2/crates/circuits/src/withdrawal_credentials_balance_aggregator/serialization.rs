use plonky2::{
    iop::target::BoolTarget,
    util::serialization::{Buffer, IoResult, Read, Write},
};

use crate::{
    serialization::targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        biguint::BigUintTarget,
        hashing::validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
    },
};

use super::first_level::circuit::ValidatorBalanceVerificationTargets;

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS: usize> ReadTargets
    for ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn read_targets(data: &mut Buffer) -> IoResult<Self> {
        Ok(ValidatorBalanceVerificationTargets {
            range_total_value: BigUintTarget::read_targets(data).unwrap(),
            range_balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            range_validator_commitment: data.read_target_hash().unwrap(),
            validators: [(); VALIDATORS_COUNT]
                .map(|_| ValidatorPoseidonTargets::read_targets(data).unwrap()),
            non_zero_validator_leaves_mask: data
                .read_target_array::<VALIDATORS_COUNT>()
                .unwrap()
                .map(|target| BoolTarget::new_unsafe(target)),
            balances: [(); VALIDATORS_COUNT / 4]
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap()),
            withdrawal_credentials: [(); WITHDRAWAL_CREDENTIALS]
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap()),
            current_epoch: BigUintTarget::read_targets(data).unwrap(),
            number_of_non_activated_validators: data.read_target().unwrap(),
            number_of_active_validators: data.read_target().unwrap(),
            number_of_exited_validators: data.read_target().unwrap(),
        })
    }
}

// TODO: This is probably wrong now
impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS: usize> WriteTargets
    for ValidatorBalanceVerificationTargets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.extend(BigUintTarget::write_targets(&self.range_total_value)?);
        data.write_target_bool_vec(&self.range_balances_root)?;
        data.write_target_hash(&self.range_validator_commitment)?;

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        data.write_target_array(&self.non_zero_validator_leaves_mask.map(|bool| bool.target))
            .unwrap();

        for balance in &self.balances {
            data.write_target_bool_vec(balance)?;
        }

        for i in 0..WITHDRAWAL_CREDENTIALS {
            data.write_target_bool_vec(&self.withdrawal_credentials[i])?;
        }

        data.extend(BigUintTarget::write_targets(&self.current_epoch)?);

        data.write_target(self.number_of_non_activated_validators)?;
        data.write_target(self.number_of_active_validators)?;
        data.write_target(self.number_of_exited_validators)?;

        Ok(data)
    }
}
