use plonky2::iop::witness::PartialWitness;

use crate::{
    circuit_input_common::{set_boolean_pw_values, SetPWValues, ValidatorBalancesInput},
    traits::WitnessSetter,
    utils::biguint::WitnessBigUint,
    withdrawal_credentials_balance_aggregator::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};

impl<const WITHDRAWAL_CREDENTIALS_COUNT: usize> WitnessSetter
    for WithdrawalCredentialsBalanceAggregatorFirstLevel<WITHDRAWAL_CREDENTIALS_COUNT>
{
    type WitnessInput = ValidatorBalancesInput;

    fn set_witness(
        targets: &Self::Targets,
        source: &Self::WitnessInput,
    ) -> PartialWitness<Self::F> {
        let mut pw = PartialWitness::new();

        for i in 0..targets.balances.len() {
            set_boolean_pw_values(&mut pw, &targets.balances[i], &source.balances[i]);
        }

        for i in 0..targets.validators.len() {
            targets.validators[i].set_pw_values(&mut pw, &source.validators[i]);
        }

        for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
            set_boolean_pw_values(
                &mut pw,
                &targets.withdrawal_credentials[i],
                &source.withdrawal_credentials[i],
            );
        }

        set_boolean_pw_values(
            &mut pw,
            &targets.non_zero_validator_leaves_mask,
            &source.validator_is_zero,
        );

        pw.set_biguint_target(&targets.current_epoch, &source.current_epoch);
        pw
    }
}
