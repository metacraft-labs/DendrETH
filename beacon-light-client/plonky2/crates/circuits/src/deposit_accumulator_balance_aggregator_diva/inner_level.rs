use crate::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    deposits_accumulator_balance_aggregator::common_targets::ValidatorStatusStatsTarget,
    utils::circuit::{
        assert_bool_arrays_are_equal, hashing::poseidon::poseidon_pair, verify_proof,
    },
};
use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions, targets::uint::ops::arithmetic::Add,
    Circuit, CircuitOutputTarget,
};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};

use super::first_level::{
    DepositAccumulatorBalanceAggregatorDivaFirstLevel, DivaAccumulatedDataTarget,
};

pub struct DepositAccumulatorBalanceAggregatorDivaInnerLevel;

const D: usize = 2;

impl Circuit for DepositAccumulatorBalanceAggregatorDivaInnerLevel {
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = D;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BasicRecursiveInnerCircuitTarget;
    type Params = CircuitData<Self::F, Self::C, D>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, D>,
        circuit_data: &Self::Params,
    ) -> Self::Target {
        let proof1 = verify_proof(builder, &circuit_data);
        let proof2 = verify_proof(builder, &circuit_data);

        let l_input = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs_target(
            &proof1.public_inputs,
        );

        let r_input = DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs_target(
            &proof2.public_inputs,
        );

        connect_pass_through_data(builder, &l_input, &r_input);

        let accumulated_data = accumulate_data(builder, &l_input, &r_input);

        let pubkey_commitment_mapper_root = poseidon_pair(
            builder,
            l_input.pubkey_commitment_mapper_root,
            r_input.pubkey_commitment_mapper_root,
        );

        let output_targets =
            CircuitOutputTarget::<DepositAccumulatorBalanceAggregatorDivaFirstLevel> {
                accumulated_data,
                pubkey_commitment_mapper_root,
                current_epoch: l_input.current_epoch,
                balances_root: l_input.balances_root,
                validators_commitment_mapper_root: l_input.validators_commitment_mapper_root,
            };

        output_targets.register_public_inputs(builder);

        Self::Target { proof1, proof2 }
    }
}

fn accumulate_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
) -> DivaAccumulatedDataTarget {
    DivaAccumulatedDataTarget {
        balance: left_range
            .accumulated_data
            .balance
            .add(right_range.accumulated_data.balance, builder),
        validator_status_stats: accumulate_validator_stats(
            builder,
            &left_range.accumulated_data.validator_status_stats,
            &right_range.accumulated_data.validator_status_stats,
        ),
    }
}

fn accumulate_validator_stats<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &ValidatorStatusStatsTarget,
    right: &ValidatorStatusStatsTarget,
) -> ValidatorStatusStatsTarget {
    return ValidatorStatusStatsTarget {
        non_activated_count: builder.add(left.non_activated_count, right.non_activated_count),
        active_count: builder.add(left.active_count, right.active_count),
        exited_count: builder.add(left.exited_count, right.exited_count),
        slashed_count: builder.add(left.slashed_count, right.slashed_count),
    };
}

fn connect_pass_through_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    l_input: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    r_input: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
) {
    builder.assert_targets_are_equal(&l_input.current_epoch, &r_input.current_epoch);
    builder.connect_hashes(
        l_input.validators_commitment_mapper_root,
        r_input.validators_commitment_mapper_root,
    );
    assert_bool_arrays_are_equal(builder, &l_input.balances_root, &r_input.balances_root);
}
