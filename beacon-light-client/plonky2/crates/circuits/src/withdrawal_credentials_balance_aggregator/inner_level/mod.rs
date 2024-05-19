use crate::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    utils::{
        biguint::CircuitBuilderBiguint,
        hashing::{
            poseidon::poseidon_pair,
            sha256::{connect_bool_arrays, sha256_pair},
        },
    },
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{Circuit, CircuitOutputTarget};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
    },
};

const D: usize = 2;

pub struct WithdrawalCredentialsBalanceAggregatorInnerLevel<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
    [(); VALIDATORS_COUNT / 4]:, {}

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
        let verifier_circuit_target = VerifierCircuitTarget {
            constants_sigmas_cap: builder
                .constant_merkle_cap(&circuit_data.verifier_only.constants_sigmas_cap),
            circuit_digest: builder.constant_hash(circuit_data.verifier_only.circuit_digest),
        };

        let proof1 = builder.add_virtual_proof_with_pis(&circuit_data.common);
        let proof2 = builder.add_virtual_proof_with_pis(&circuit_data.common);

        builder.verify_proof::<Self::C>(&proof1, &verifier_circuit_target, &circuit_data.common);
        builder.verify_proof::<Self::C>(&proof2, &verifier_circuit_target, &circuit_data.common);

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

        let number_of_exitted_validators = builder.add(
            l_input.number_of_exitted_validators,
            r_input.number_of_exitted_validators,
        );

        let mut range_total_value =
            builder.add_biguint(&l_input.range_total_value, &r_input.range_total_value);

        // pop carry
        range_total_value.limbs.pop();

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
            range_total_value,
            range_balances_root,
            withdrawal_credentials: l_input.withdrawal_credentials,
            range_validator_commitment,
            number_of_non_activated_validators,
            number_of_active_validators,
            number_of_exitted_validators,
        };

        output_target.register_public_inputs(builder);

        Self::Target {
            proof1,
            proof2,
            verifier_circuit_target,
        }
    }
}
