use itertools::Itertools;
use plonky2::util::serialization::{Buffer, IoResult, Read, Write};

use crate::{
    serialization::targets_serialization::{ReadTargets, WriteTargets},
    utils::{
        biguint::BigUintTarget,
        hashing::validator_hash_tree_root_poseidon::ValidatorPoseidonTargets,
    },
};

use super::first_level::circuit::ValidatorBalanceVerificationTargets;

impl<const N: usize> ReadTargets for ValidatorBalanceVerificationTargets<N> {
    fn read_targets(data: &mut Buffer) -> IoResult<ValidatorBalanceVerificationTargets<N>> {
        let validators_len = data.read_usize()?;

        Ok(ValidatorBalanceVerificationTargets {
            range_total_value: BigUintTarget::read_targets(data)?,
            range_balances_root: data.read_target_bool_vec()?.try_into().unwrap(),
            range_validator_commitment: data.read_target_hash()?,
            validators: (0..validators_len)
                .map(|_| ValidatorPoseidonTargets::read_targets(data).unwrap())
                .collect(),
            non_zero_validator_leaves_mask: data.read_target_bool_vec()?,
            balances: (0..validators_len / 4)
                .map(|_| data.read_target_bool_vec().unwrap().try_into().unwrap())
                .collect(),
            withdrawal_credentials: (0..N)
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

impl<const N: usize> WriteTargets for ValidatorBalanceVerificationTargets<N> {
    fn write_targets(&self) -> IoResult<Vec<u8>> {
        let mut data = Vec::<u8>::new();

        data.write_usize(self.validators.len())?;
        data.extend(BigUintTarget::write_targets(&self.range_total_value)?);
        data.write_target_bool_vec(&self.range_balances_root)?;
        data.write_target_hash(&self.range_validator_commitment)?;

        for validator in &self.validators {
            data.extend(ValidatorPoseidonTargets::write_targets(validator)?);
        }

        data.write_target_bool_vec(&self.non_zero_validator_leaves_mask)?;

        for balance in &self.balances {
            data.write_target_bool_vec(balance)?;
        }

        for i in 0..N {
            data.write_target_bool_vec(&self.withdrawal_credentials[i])?;
        }

        data.extend(BigUintTarget::write_targets(&self.current_epoch)?);

        data.write_target(self.number_of_non_activated_validators)?;
        data.write_target(self.number_of_active_validators)?;
        data.write_target(self.number_of_exited_validators)?;

        Ok(data)
    }
}
