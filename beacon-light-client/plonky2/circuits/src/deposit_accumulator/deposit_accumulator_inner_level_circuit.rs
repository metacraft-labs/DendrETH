use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifierCircuitTarget},
        config::{GenericConfig, PoseidonGoldilocksConfig},
    },
};

use crate::{
    biguint::CircuitBuilderBiguint,
    deposit_accumulator::{
        traits::DepositAccumulatorNodeTargetExt, utils::set_node_public_variables,
    },
    utils::{biguint_is_equal, bits_to_biguint_target, if_biguint},
};

use super::objects::{AccumulatedDataTargets, NodeTargets, RangeObject, ValidatorStatsTargets};

/// Returns `BoolTarget` - true if pk1 <= pl2
pub fn cmp_pubkey<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pk1: [BoolTarget; 384],
    pk2: [BoolTarget; 384],
) -> BoolTarget {
    let pk1_bu = bits_to_biguint_target(builder, pk1.to_vec());
    let pk2_bu = bits_to_biguint_target(builder, pk2.to_vec());

    builder.cmp_biguint(&pk1_bu, &pk2_bu)
}

pub fn calc_counted_data<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    left: &NodeTargets,
    right: &NodeTargets,
) -> (BoolTarget, BoolTarget) {
    let x = left.leftmost.pubkey;

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

pub fn inherit_bounds_data_from_children<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    left: &NodeTargets,
    right: &NodeTargets,
) -> (RangeObject, RangeObject) {
    let (left_counted_data, right_counted_data) = calc_counted_data(builder, left, right);

    let left_bound = RangeObject {
        pubkey: left.leftmost.pubkey, //TODO: Review Validator in ref?
        deposit_index: left.leftmost.deposit_index.clone(),
        is_counted: left_counted_data,
        is_dummy: left.leftmost.is_dummy,
    };

    let right_bound = RangeObject {
        pubkey: right.rightmost.pubkey,
        deposit_index: right.rightmost.deposit_index.clone(),
        is_counted: right_counted_data,
        is_dummy: right.rightmost.is_dummy,
    };

    (left_bound, right_bound)
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
        non_activated_validators_count: non_activated_validators_count,
        active_validators_count: active_validators_count,
        exited_validators_count: exited_validators_count,
        slashed_validators_count: slashed_validators_count,
    };
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

pub fn is_zero_proof<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    node: &NodeTargets,
) -> BoolTarget {
    let _true = builder._true();
    let is_leftmost_fictional = builder.and(node.leftmost.is_dummy, _true);
    let is_rightmost_fictional = builder.and(node.rightmost.is_dummy, _true);

    builder.and(is_leftmost_fictional, is_rightmost_fictional)
}

pub fn are_pubkeys_equal<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    pk1: &[BoolTarget; 384],
    pk2: &[BoolTarget; 384],
) -> BoolTarget {
    let mut pubkeys_are_same = builder._true();

    for i in 0..384 {
        let is_bit_equal = builder.and(pk1[i], pk2[i]);
        pubkeys_are_same = builder.and(is_bit_equal, pubkeys_are_same);
    }

    pubkeys_are_same
}

pub fn has_same_pubkey_and_is_counted<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    pubkey: &[BoolTarget; 384],
    data: &RangeObject,
) -> BoolTarget {
    let are_pubkeys_same = are_pubkeys_equal(builder, pubkey, &data.pubkey);

    builder.and(data.is_counted, are_pubkeys_same)
}

pub fn pubkeys_are_same_and_are_counted<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
    builder: &mut CircuitBuilder<F, D>,
    x: &RangeObject,
    y: &RangeObject,
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

    let balance_final = if_biguint(
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

pub fn connect_pass_through_data<F: RichField + Extendable<D>, const D: usize>(
    //TODO: Review
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
        left_node.commitment_mapper_proof_root,
        right_node.commitment_mapper_proof_root,
    );
    builder.connect_hashes(
        left_node.merkle_tree_deposit_branch_root,
        right_node.merkle_tree_deposit_branch_root,
    );
}

pub fn build_commitment_mapper_inner_circuit(
    inner_circuit_data: &CircuitData<
        plonky2::field::goldilocks_field::GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >,
) {
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let mut builder = CircuitBuilder::<F, D>::new(standard_recursion_config);

    let verifier_circuit_target = VerifierCircuitTarget {
        constants_sigmas_cap: builder
            .constant_merkle_cap(&inner_circuit_data.verifier_only.constants_sigmas_cap),
        circuit_digest: builder.constant_hash(inner_circuit_data.verifier_only.circuit_digest),
    };

    let left = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);
    let right = builder.add_virtual_proof_with_pis(&inner_circuit_data.common);

    builder.verify_proof::<C>(&left, &verifier_circuit_target, &inner_circuit_data.common);
    builder.verify_proof::<C>(&right, &verifier_circuit_target, &inner_circuit_data.common);

    let left_node = left.get_node();
    let right_node = right.get_node();

    let _true = builder._true();

    connect_pass_through_data(&mut builder, &left_node, &right_node);

    let pk_are_monotonic_ordering = cmp_pubkey(
        &mut builder,
        left_node.rightmost.pubkey,
        right_node.leftmost.pubkey,
    );
    builder.connect(pk_are_monotonic_ordering.target, _true.target);

    let right_is_zero_proof = is_zero_proof(&mut builder, &right_node);

    let inner_deposits_are_monotonic = builder.cmp_biguint(
        &left_node.rightmost.deposit_index,
        &right_node.leftmost.deposit_index,
    );

    let inner_deposits_are_equal = biguint_is_equal(
        &mut builder,
        &left_node.rightmost.deposit_index,
        &right_node.leftmost.deposit_index,
    );
    let inner_deposits_are_not_equal = builder.not(inner_deposits_are_equal);
    let inner_deposits_are_strictly_monotonic =
        builder.and(inner_deposits_are_monotonic, inner_deposits_are_not_equal);
    let inner_deposits_are_strictly_monotonic_or_dummy =
        builder.or(inner_deposits_are_strictly_monotonic, right_is_zero_proof);
    builder.connect(
        inner_deposits_are_strictly_monotonic_or_dummy.target,
        _true.target,
    );

    let (left_bound, right_bound) =
        inherit_bounds_data_from_children(&mut builder, &left_node, &right_node);
    let accumulated_pre_discount = accumulate_data(&mut builder, &left_node, &right_node);
    let accumulated_final = account_for_double_counting(
        &mut builder,
        accumulated_pre_discount,
        &left_node,
        &right_node,
    );

    let parent = NodeTargets {
        leftmost: left_bound,
        rightmost: right_bound,
        accumulated: accumulated_final,

        // Pass through
        current_epoch: left_node.current_epoch,
        eth1_deposit_index: left_node.eth1_deposit_index,
        commitment_mapper_proof_root: left_node.commitment_mapper_proof_root,
        merkle_tree_deposit_branch_root: left_node
            .merkle_tree_deposit_branch_root,
    };

    set_node_public_variables(&mut builder, &parent);
}
