use crate::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    utils::circuit::{
        connect_bool_arrays,
        hashing::{poseidon::poseidon_pair, sha256::sha256_pair},
        verify_proof,
    },
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions, targets::uint::ops::arithmetic::Add,
    Circuit, CircuitOutputTarget,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};

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

        let number_of_non_activated_validators = builder.add(
            l_input.number_of_non_activated_validators,
            r_input.number_of_non_activated_validators,
        );

        let number_of_active_validators = builder.add(
            l_input.number_of_active_validators,
            r_input.number_of_active_validators,
        );

        let number_of_exited_validators = builder.add(
            l_input.number_of_exited_validators,
            r_input.number_of_exited_validators,
        );

        let number_of_slashed_validators = builder.add(
            l_input.number_of_slashed_validators,
            r_input.number_of_slashed_validators,
        );

        let range_total_value = l_input
            .range_total_value
            .add(r_input.range_total_value, builder);

        for i in 0..WITHDRAWAL_CREDENTIALS_COUNT {
            connect_bool_arrays(
                builder,
                &l_input.withdrawal_credentials[i],
                &r_input.withdrawal_credentials[i],
            );
        }

        builder.assert_targets_are_equal(&l_input.current_epoch, &r_input.current_epoch);

        let output_target = CircuitOutputTarget::<
            WithdrawalCredentialsBalanceAggregatorFirstLevel<
                VALIDATORS_COUNT,
                WITHDRAWAL_CREDENTIALS_COUNT,
            >,
        > {
            current_epoch: l_input.current_epoch,
            range_total_value,
            range_balances_root,
            withdrawal_credentials: l_input.withdrawal_credentials,
            range_validator_commitment,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exited_validators,
            number_of_slashed_validators,
        };

        output_target.register_public_inputs(builder);

        Self::Target { proof1, proof2 }
    }
}
