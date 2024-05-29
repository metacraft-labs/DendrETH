use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit, CircuitOutputTarget};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_crypto::biguint::CircuitBuilderBiguint;

use crate::{
    common_targets::{BasicRecursiveInnerCircuitTarget, PubkeyTarget},
    deposits_accumulator_balance_aggregator::first_level::DepositAccumulatorBalanceAggregatorFirstLevel,
    utils::circuit::{
        assert_bool_arrays_are_equal, biguint_is_equal, bits_to_biguint_target,
        bool_arrays_are_equal, verify_proof,
    },
};

use super::common_targets::{AccumulatedDataTarget, DepositDataTarget, ValidatorStatusStatsTarget};

pub struct DepositAccumulatorBalanceAggregatorInnerLevel {}

impl Circuit for DepositAccumulatorBalanceAggregatorInnerLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = BasicRecursiveInnerCircuitTarget;

    type Params = CircuitData<Self::F, Self::C, { Self::D }>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        circuit_data: &Self::Params,
    ) -> Self::Target {
        let left_range_proof = verify_proof(builder, &circuit_data);
        let right_range_proof = verify_proof(builder, &circuit_data);

        let left_range = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &left_range_proof.public_inputs,
        );

        let right_range = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &right_range_proof.public_inputs,
        );

        connect_pass_through_data(builder, &left_range, &right_range);
        assert_no_dummy_deposits_to_the_left(builder, &left_range, &right_range);
        assert_deposits_are_sorted(builder, &left_range, &right_range);

        let (leftmost_deposit, rightmost_deposit) =
            inherit_bounding_deposits_data_from_children(builder, &left_range, &right_range);

        let accumulated_pre_discount = accumulate_data(builder, &left_range, &right_range);
        let accumulated_data = account_for_double_counting(
            builder,
            accumulated_pre_discount,
            &left_range,
            &right_range,
        );

        let output_targets = CircuitOutputTarget::<DepositAccumulatorBalanceAggregatorFirstLevel> {
            leftmost_deposit,
            rightmost_deposit,
            accumulated_data,
            deposits_commitment_mapper_root: left_range.deposits_commitment_mapper_root,
            current_epoch: left_range.current_epoch,
            eth1_deposit_index: left_range.eth1_deposit_index,
            commitment_mapper_root: left_range.commitment_mapper_root,
            balances_root: left_range.balances_root,
            genesis_fork_version: left_range.genesis_fork_version,
        };

        output_targets.register_public_inputs(builder);

        Self::Target {
            proof1: left_range_proof,
            proof2: right_range_proof,
        }
    }
}

fn connect_pass_through_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) {
    builder.connect_biguint(&left_range.current_epoch, &right_range.current_epoch);
    builder.connect_biguint(
        &left_range.eth1_deposit_index,
        &right_range.eth1_deposit_index,
    );
    builder.connect_hashes(
        left_range.commitment_mapper_root,
        right_range.commitment_mapper_root,
    );
    builder.connect_hashes(
        left_range.deposits_commitment_mapper_root,
        right_range.deposits_commitment_mapper_root,
    );
    assert_bool_arrays_are_equal(
        builder,
        &left_range.balances_root,
        &right_range.balances_root,
    );
    assert_bool_arrays_are_equal(
        builder,
        &left_range.genesis_fork_version,
        &right_range.genesis_fork_version,
    );
}

/// Returns `BoolTarget` - true if pk1 <= pl2
fn cmp_pubkey<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pk1: PubkeyTarget,
    pk2: PubkeyTarget,
) -> BoolTarget {
    let pk1_bu = bits_to_biguint_target(builder, pk1.to_vec());
    let pk2_bu = bits_to_biguint_target(builder, pk2.to_vec());

    builder.cmp_biguint(&pk1_bu, &pk2_bu)
}

fn inherit_bounding_deposits_data_from_children<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) -> (DepositDataTarget, DepositDataTarget) {
    let (left_is_counted, right_is_counted) = calc_counted_data(builder, left_range, right_range);

    let leftmost_deposit = DepositDataTarget {
        validator: left_range.leftmost_deposit.validator.clone(),
        pubkey: left_range.leftmost_deposit.pubkey,
        deposit_index: left_range.leftmost_deposit.deposit_index.clone(),
        is_counted: left_is_counted,
        is_dummy: left_range.leftmost_deposit.is_dummy,
    };

    let rightmost_deposit = DepositDataTarget {
        validator: right_range.rightmost_deposit.validator.clone(),
        pubkey: right_range.rightmost_deposit.pubkey,
        deposit_index: right_range.rightmost_deposit.deposit_index.clone(),
        is_counted: right_is_counted,
        is_dummy: right_range.rightmost_deposit.is_dummy,
    };

    (leftmost_deposit, rightmost_deposit)
}

fn calc_counted_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) -> (BoolTarget, BoolTarget) {
    let leftmost_pubkey = left_range.leftmost_deposit.pubkey;
    let l_x = deposit_has_same_pubkey_and_counted(
        builder,
        &leftmost_pubkey,
        &left_range.rightmost_deposit,
    );
    let l_y = deposit_has_same_pubkey_and_counted(
        builder,
        &leftmost_pubkey,
        &right_range.leftmost_deposit,
    );
    let l_z = deposit_has_same_pubkey_and_counted(
        builder,
        &leftmost_pubkey,
        &right_range.rightmost_deposit,
    );

    let mut is_leftmost_counted = left_range.leftmost_deposit.is_counted;
    is_leftmost_counted = builder.or(is_leftmost_counted, l_x);
    is_leftmost_counted = builder.or(is_leftmost_counted, l_y);
    is_leftmost_counted = builder.or(is_leftmost_counted, l_z);

    let rightmost_pubkey = right_range.rightmost_deposit.pubkey;
    let r_x = deposit_has_same_pubkey_and_counted(
        builder,
        &rightmost_pubkey,
        &left_range.leftmost_deposit,
    );
    let r_y = deposit_has_same_pubkey_and_counted(
        builder,
        &rightmost_pubkey,
        &left_range.rightmost_deposit,
    );
    let r_z = deposit_has_same_pubkey_and_counted(
        builder,
        &rightmost_pubkey,
        &right_range.leftmost_deposit,
    );

    let mut is_rightmost_counted = right_range.rightmost_deposit.is_counted;
    is_rightmost_counted = builder.or(is_rightmost_counted, r_x);
    is_rightmost_counted = builder.or(is_rightmost_counted, r_y);
    is_rightmost_counted = builder.or(is_rightmost_counted, r_z);

    return (is_leftmost_counted, is_rightmost_counted);
}

fn deposit_has_same_pubkey_and_counted<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pubkey: &PubkeyTarget,
    deposit: &DepositDataTarget,
) -> BoolTarget {
    let pubkeys_are_same = bool_arrays_are_equal(builder, pubkey, &deposit.pubkey);
    builder.and(pubkeys_are_same, deposit.is_counted)
}

fn accumulate_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) -> AccumulatedDataTarget {
    AccumulatedDataTarget {
        balance: builder.add_biguint(
            &left_range.accumulated_data.balance,
            &right_range.accumulated_data.balance,
        ),
        deposits_count: builder.add(
            left_range.accumulated_data.deposits_count,
            right_range.accumulated_data.deposits_count,
        ),
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

fn pubkeys_are_same_and_counted<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: &DepositDataTarget,
    y: &DepositDataTarget,
) -> BoolTarget {
    let pubkeys_are_same = bool_arrays_are_equal(builder, &x.pubkey, &y.pubkey);
    let both_are_counted = builder.and(x.is_counted, y.is_counted);
    builder.and(pubkeys_are_same, both_are_counted)
}

fn account_for_double_counting<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    accumulated_data: AccumulatedDataTarget,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) -> AccumulatedDataTarget {
    let new_accumulated_data = AccumulatedDataTarget {
        balance: builder.sub_biguint(
            &accumulated_data.balance,
            &left_range.accumulated_data.balance,
        ),
        deposits_count: accumulated_data.deposits_count,
        validator_status_stats: ValidatorStatusStatsTarget {
            non_activated_count: builder.sub(
                accumulated_data.validator_status_stats.non_activated_count,
                left_range
                    .leftmost_deposit
                    .validator
                    .is_non_activated
                    .target,
            ),
            active_count: builder.sub(
                accumulated_data.validator_status_stats.active_count,
                left_range.leftmost_deposit.validator.is_active.target,
            ),
            exited_count: builder.sub(
                accumulated_data.validator_status_stats.exited_count,
                left_range.leftmost_deposit.validator.is_exited.target,
            ),
            slashed_count: builder.sub(
                accumulated_data.validator_status_stats.slashed_count,
                left_range.leftmost_deposit.validator.is_slashed.target,
            ),
        },
    };

    let are_pubkeys_same_and_counted = pubkeys_are_same_and_counted(
        builder,
        &left_range.rightmost_deposit,
        &right_range.leftmost_deposit,
    );

    builder.select_target(
        are_pubkeys_same_and_counted,
        &new_accumulated_data,
        &accumulated_data,
    )
}

fn assert_deposits_are_sorted<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) {
    let pks_are_increasing = cmp_pubkey(
        builder,
        left_range.rightmost_deposit.pubkey,
        right_range.leftmost_deposit.pubkey,
    );

    let pks_are_same = bool_arrays_are_equal(
        builder,
        &left_range.rightmost_deposit.pubkey,
        &right_range.leftmost_deposit.pubkey,
    );

    let deposits_are_same = biguint_is_equal(
        builder,
        &left_range.rightmost_deposit.deposit_index,
        &right_range.leftmost_deposit.deposit_index,
    );

    let deposits_are_different = builder.not(deposits_are_same);

    let deposits_are_increasing = builder.cmp_biguint(
        &left_range.rightmost_deposit.deposit_index,
        &right_range.leftmost_deposit.deposit_index,
    );

    let deposits_are_strictly_increasing =
        builder.and(deposits_are_increasing, deposits_are_different);

    let if_pks_are_same_then_deposits_are_strictly_increasing =
        builder.imply(pks_are_same, deposits_are_strictly_increasing);

    let ordering_is_respected = builder.and(
        pks_are_increasing,
        if_pks_are_same_then_deposits_are_strictly_increasing,
    );

    let ordering_is_respected_or_right_is_dummy =
        builder.or(ordering_is_respected, right_range.leftmost_deposit.is_dummy);

    builder.assert_true(ordering_is_respected_or_right_is_dummy);
}

fn assert_no_dummy_deposits_to_the_left<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    right_range: &CircuitOutputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
) {
    builder.assert_implication(
        left_range.rightmost_deposit.is_dummy,
        right_range.leftmost_deposit.is_dummy,
    );
}

#[cfg(test)]
mod test {}
