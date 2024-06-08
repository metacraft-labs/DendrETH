use crate::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    utils::circuit::{
        connect_bool_arrays,
        hashing::{poseidon::poseidon_pair, sha256::sha256_pair},
        verify_proof,
    },
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{Circuit, CircuitOutputTarget};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_crypto::biguint::CircuitBuilderBiguint;

use super::first_level::AccumulatedValidatorsData;

pub struct WithdrawalCredentialsBalanceAggregatorInnerLevel<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:;

const D: usize = 2;

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> Circuit
    for WithdrawalCredentialsBalanceAggregatorInnerLevel<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BasicRecursiveInnerCircuitTarget;
    type Params = CircuitData<Self::F, Self::C, D>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, D>,
        circuit_data: &Self::Params,
    ) -> Self::Target where {
        let proof1 = verify_proof(builder, &circuit_data);
        let proof2 = verify_proof(builder, &circuit_data);

        let l_input = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >::read_public_inputs_target(&proof1.public_inputs);

        let r_input = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >::read_public_inputs_target(&proof2.public_inputs);

        let range_validator_commitment = poseidon_pair(
            builder,
            l_input.range_validator_commitment,
            r_input.range_validator_commitment,
        );

        let range_balances_root = sha256_pair(
            builder,
            &l_input.range_balances_root,
            &r_input.range_balances_root,
        );

        let mut accumulated_balance = builder.add_biguint(
            &l_input.accumulated_data.balance,
            &r_input.accumulated_data.balance,
        );
        accumulated_balance.limbs.pop();

        let accumulated_data = AccumulatedValidatorsData {
            balance: accumulated_balance,
            non_activated_count: builder.add(
                l_input.accumulated_data.non_activated_count,
                r_input.accumulated_data.non_activated_count,
            ),
            active_count: builder.add(
                l_input.accumulated_data.active_count,
                r_input.accumulated_data.active_count,
            ),
            exited_count: builder.add(
                l_input.accumulated_data.exited_count,
                r_input.accumulated_data.exited_count,
            ),
            slashed_count: builder.add(
                l_input.accumulated_data.slashed_count,
                r_input.accumulated_data.slashed_count,
            ),
        };

        for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
            connect_bool_arrays(
                builder,
                &l_input.withdrawal_credentials[i],
                &r_input.withdrawal_credentials[i],
            );
        }

        builder.connect_biguint(&l_input.current_epoch, &r_input.current_epoch);

        let output_target = CircuitOutputTarget::<
            WithdrawalCredentialsBalanceAggregatorFirstLevel<
                VALIDATORS_COUNT,
                WITHDRAWAL_CREDENTIALS_COUNT,
            >,
        > {
            current_epoch: l_input.current_epoch,
            range_balances_root,
            withdrawal_credentials: l_input.withdrawal_credentials,
            range_validator_commitment,
            accumulated_data,
        };

        output_target.register_public_inputs(builder);

        Self::Target { proof1, proof2 }
    }
}
