use circuit::{circuit_builder_extensions::CircuitBuilderExtensions, Circuit, CircuitOutputTarget};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_crypto::biguint::CircuitBuilderBiguint;

use crate::{
    common_targets::{BasicRecursiveInnerCircuitTarget, PubkeyTarget},
    deposits_accumulator_balance_aggregator::first_level::DepositAccumulatorBalanceAggregatorFirstLevel,
    utils::circuit::{biguint_is_equal, bits_to_biguint_target, select_biguint},
};

use super::common_targets::{
    AccumulatedDataTargets, NodeTargets, RangeObjectTarget, ValidatorStatsTargets,
};

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
        let verifier_circuit_target = VerifierCircuitTarget {
            constants_sigmas_cap: builder
                .constant_merkle_cap(&circuit_data.verifier_only.constants_sigmas_cap),
            circuit_digest: builder.constant_hash(circuit_data.verifier_only.circuit_digest),
        };

        let proof1 = builder.add_virtual_proof_with_pis(&circuit_data.common);
        let proof2 = builder.add_virtual_proof_with_pis(&circuit_data.common);

        builder.verify_proof::<Self::C>(&proof1, &verifier_circuit_target, &circuit_data.common);
        builder.verify_proof::<Self::C>(&proof2, &verifier_circuit_target, &circuit_data.common);

        let left_node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &proof1.public_inputs,
        )
        .node;

        let right_node = DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs_target(
            &proof2.public_inputs,
        )
        .node;

        let _true = builder._true();
        let _false = builder._false();

        assert_no_dummy_deposits_to_the_left(builder, &left_node, &right_node);

        connect_pass_through_data(builder, &left_node, &right_node);

        let mut pk_are_monotonic_ordering = cmp_pubkey(
            builder,
            left_node.rightmost.pubkey,
            right_node.leftmost.pubkey,
        );

        pk_are_monotonic_ordering =
            builder.or(pk_are_monotonic_ordering, right_node.leftmost.is_dummy);

        builder.assert_true(pk_are_monotonic_ordering);

        let inner_deposits_are_monotonic = builder.cmp_biguint(
            &left_node.rightmost.deposit_index,
            &right_node.leftmost.deposit_index,
        );

        let inner_deposits_are_equal = biguint_is_equal(
            builder,
            &left_node.rightmost.deposit_index,
            &right_node.leftmost.deposit_index,
        );
        builder.connect(inner_deposits_are_equal.target, _false.target);
        let inner_deposits_are_not_equal = builder.not(inner_deposits_are_equal);
        let inner_deposits_are_strictly_monotonic =
            builder.and(inner_deposits_are_monotonic, inner_deposits_are_not_equal);
        let inner_deposits_are_strictly_monotonic_or_dummy = builder.or(
            inner_deposits_are_strictly_monotonic,
            right_node.leftmost.is_dummy,
        );
        let pks_are_equal = are_pubkeys_equal(
            builder,
            &left_node.rightmost.pubkey,
            &right_node.leftmost.pubkey,
        );

        let should_equal = builder._if(
            pks_are_equal,
            _true.target,
            inner_deposits_are_strictly_monotonic_or_dummy.target,
        );

        builder.connect(
            should_equal,
            inner_deposits_are_strictly_monotonic_or_dummy.target,
        );

        let (left_bound, right_bound) =
            inherit_bounds_data_from_children(builder, &left_node, &right_node);
        let accumulated_pre_discount = accumulate_data(builder, &left_node, &right_node);

        let accumulated_final =
            account_for_double_counting(builder, accumulated_pre_discount, &left_node, &right_node);

        let parent = NodeTargets {
            leftmost: left_bound,
            rightmost: right_bound,
            accumulated: accumulated_final,

            // Pass through
            current_epoch: left_node.current_epoch,
            eth1_deposit_index: left_node.eth1_deposit_index,
            commitment_mapper_root: left_node.commitment_mapper_root,
            deposits_mapper_root: left_node.commitment_mapper_root,
            balances_root: left_node.balances_root,
            genesis_fork_version: left_node.genesis_fork_version,
        };

        let output_targets =
            CircuitOutputTarget::<DepositAccumulatorBalanceAggregatorFirstLevel> { node: parent };

        output_targets.register_public_inputs(builder);

        Self::Target { proof1, proof2 }
    }
}

pub fn connect_pass_through_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_node: &NodeTargets,
    right_node: &NodeTargets,
) {
    builder.connect_biguint(&left_node.current_epoch, &right_node.current_epoch);
    builder.connect_biguint(
        &left_node.eth1_deposit_index,
        &right_node.eth1_deposit_index,
    );
    builder.connect_hashes(
        left_node.commitment_mapper_root,
        right_node.commitment_mapper_root,
    );
    builder.connect_hashes(
        left_node.deposits_mapper_root,
        right_node.deposits_mapper_root,
    );

    for i in 0..256 {
        builder.connect(
            left_node.balances_root[i].target,
            right_node.balances_root[i].target,
        );
    }

    for i in 0..32 {
        builder.connect(
            left_node.genesis_fork_version[i].target,
            right_node.genesis_fork_version[i].target,
        );
    }
}

/// Returns `BoolTarget` - true if pk1 <= pl2
pub fn cmp_pubkey<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pk1: PubkeyTarget,
    pk2: PubkeyTarget,
) -> BoolTarget {
    let pk1_bu = bits_to_biguint_target(builder, pk1.to_vec());
    let pk2_bu = bits_to_biguint_target(builder, pk2.to_vec());

    builder.cmp_biguint(&pk1_bu, &pk2_bu)
}

pub fn are_pubkeys_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pk1: &PubkeyTarget,
    pk2: &PubkeyTarget,
) -> BoolTarget {
    let mut is_equal = builder._true();

    for i in 0..pk1.len() {
        let is_equal_i = builder.is_equal(pk1[i].target, pk2[i].target);
        is_equal = builder.and(is_equal, is_equal_i);
    }

    is_equal
}

pub fn inherit_bounds_data_from_children<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    left: &NodeTargets,
    right: &NodeTargets,
) -> (RangeObjectTarget, RangeObjectTarget) {
    let (left_counted_data, right_counted_data) = calc_counted_data(builder, left, right);

    let left_bound = RangeObjectTarget {
        pubkey: left.leftmost.pubkey, //TODO: Review Validator in ref?
        deposit_index: left.leftmost.deposit_index.clone(),
        is_counted: left_counted_data,
        is_dummy: left.leftmost.is_dummy,
    };

    let right_bound = RangeObjectTarget {
        pubkey: right.rightmost.pubkey,
        deposit_index: right.rightmost.deposit_index.clone(),
        is_counted: right_counted_data,
        is_dummy: right.rightmost.is_dummy,
    };

    (left_bound, right_bound)
}

pub fn calc_counted_data<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    left: &NodeTargets,
    right: &NodeTargets,
) -> (BoolTarget, BoolTarget) {
    let mut is_leftmost_counted = left.leftmost.is_counted;
    let l_x = has_same_pubkey_and_is_counted(builder, &left.leftmost.pubkey, &left.rightmost);
    let l_y = has_same_pubkey_and_is_counted(builder, &left.leftmost.pubkey, &right.leftmost);
    let l_z = has_same_pubkey_and_is_counted(builder, &left.leftmost.pubkey, &right.rightmost);

    is_leftmost_counted = builder.or(is_leftmost_counted, l_x);
    is_leftmost_counted = builder.or(is_leftmost_counted, l_y);
    is_leftmost_counted = builder.or(is_leftmost_counted, l_z);

    let mut is_rightmost_counted = right.rightmost.is_counted;
    let r_x = has_same_pubkey_and_is_counted(builder, &right.rightmost.pubkey, &left.leftmost);
    let r_y = has_same_pubkey_and_is_counted(builder, &right.rightmost.pubkey, &left.rightmost);
    let r_z = has_same_pubkey_and_is_counted(builder, &right.rightmost.pubkey, &right.leftmost);

    is_rightmost_counted = builder.or(is_rightmost_counted, r_x);
    is_rightmost_counted = builder.or(is_rightmost_counted, r_y);
    is_rightmost_counted = builder.or(is_rightmost_counted, r_z);

    return (is_leftmost_counted, is_rightmost_counted);
}

pub fn has_same_pubkey_and_is_counted<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    pubkey: &PubkeyTarget,
    data: &RangeObjectTarget,
) -> BoolTarget {
    let are_pubkeys_same = are_pubkeys_equal(builder, pubkey, &data.pubkey);

    builder.and(data.is_counted, are_pubkeys_same)
}

pub fn accumulate_data<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &NodeTargets,
    right: &NodeTargets,
) -> AccumulatedDataTargets {
    let balance_sum = builder.add_biguint(
        &left.accumulated.balance_sum,
        &right.accumulated.balance_sum,
    );
    let deposits_count = builder.add(
        left.accumulated.deposits_count,
        right.accumulated.deposits_count,
    );
    let validator_stats = accumulate_validator_stats(
        builder,
        &left.accumulated.validator_stats,
        &right.accumulated.validator_stats,
    );

    return AccumulatedDataTargets {
        balance_sum: balance_sum,
        deposits_count: deposits_count,
        validator_stats: validator_stats,
    };
}

pub fn accumulate_validator_stats<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &ValidatorStatsTargets,
    right: &ValidatorStatsTargets,
) -> ValidatorStatsTargets {
    let non_activated_validators_count = builder.add(
        left.non_activated_validators_count,
        right.non_activated_validators_count,
    );
    let active_validators_count =
        builder.add(left.active_validators_count, right.active_validators_count);
    let exited_validators_count =
        builder.add(left.exited_validators_count, right.exited_validators_count);
    let slashed_validators_count = builder.add(
        left.slashed_validators_count,
        right.slashed_validators_count,
    );

    return ValidatorStatsTargets {
        non_activated_validators_count,
        active_validators_count,
        exited_validators_count,
        slashed_validators_count,
    };
}

pub fn pubkeys_are_same_and_are_counted<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    x: &RangeObjectTarget,
    y: &RangeObjectTarget,
) -> BoolTarget {
    let pubkeys_are_same = are_pubkeys_equal(builder, &x.pubkey, &y.pubkey);

    let both_are_counted = builder.and(x.is_counted, y.is_counted);

    builder.and(pubkeys_are_same, both_are_counted)
}

pub fn account_for_double_counting<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    accumulated_data: AccumulatedDataTargets,
    left: &NodeTargets,
    right: &NodeTargets,
) -> AccumulatedDataTargets {
    let are_pubkeys_same_and_counted =
        pubkeys_are_same_and_are_counted(builder, &left.rightmost, &right.leftmost);

    let balance = builder.sub_biguint(&accumulated_data.balance_sum, &left.accumulated.balance_sum);

    let non_activated_validators_count = builder.sub(
        accumulated_data
            .validator_stats
            .non_activated_validators_count,
        left.accumulated
            .validator_stats
            .non_activated_validators_count,
    );

    let active_validators_count = builder.sub(
        accumulated_data.validator_stats.active_validators_count,
        left.accumulated.validator_stats.active_validators_count,
    );

    let exited_validators_count = builder.sub(
        accumulated_data.validator_stats.exited_validators_count,
        left.accumulated.validator_stats.exited_validators_count,
    );

    let slashed_validators_count = builder.sub(
        accumulated_data.validator_stats.slashed_validators_count,
        left.accumulated.validator_stats.slashed_validators_count,
    );

    let balance_final = select_biguint(
        builder,
        are_pubkeys_same_and_counted,
        &balance,
        &accumulated_data.balance_sum,
    );
    let non_activated_validators_count_final = builder._if(
        are_pubkeys_same_and_counted,
        non_activated_validators_count,
        accumulated_data
            .validator_stats
            .non_activated_validators_count,
    );
    let active_validators_count_final = builder._if(
        are_pubkeys_same_and_counted,
        active_validators_count,
        accumulated_data.validator_stats.active_validators_count,
    );
    let exited_validators_count_final = builder._if(
        are_pubkeys_same_and_counted,
        exited_validators_count,
        accumulated_data.validator_stats.exited_validators_count,
    );
    let slashed_validators_count_final = builder._if(
        are_pubkeys_same_and_counted,
        slashed_validators_count,
        accumulated_data.validator_stats.slashed_validators_count,
    );
    return AccumulatedDataTargets {
        balance_sum: balance_final,
        deposits_count: accumulated_data.deposits_count, //TODO: Not modified, Correct?
        validator_stats: ValidatorStatsTargets {
            non_activated_validators_count: non_activated_validators_count_final,
            active_validators_count: active_validators_count_final,
            exited_validators_count: exited_validators_count_final,
            slashed_validators_count: slashed_validators_count_final,
        },
    };
}

fn assert_no_dummy_deposits_to_the_left<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left_node: &NodeTargets,
    right_node: &NodeTargets,
) {
    let left_bound_is_not_dummy = builder.not(left_node.rightmost.is_dummy);
    let right_bound_is_not_dummy = builder.not(right_node.leftmost.is_dummy);
    let no_dummies_to_the_left = builder.imply(right_bound_is_not_dummy, left_bound_is_not_dummy);
    builder.assert_true(no_dummies_to_the_left)
}

#[cfg(test)]
mod test {}
