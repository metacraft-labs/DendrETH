use super::{checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable};
use crate::{
    constants::SLOTS_PER_EPOCH,
    types::{Epoch, Gwei, Slot},
    utils::plonky2x_extensions::assert_is_true,
};
use plonky2x::{
    frontend::vars::Variable,
    prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable},
};

fn validate_target<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    current_epoch: Epoch,
    target_epoch: Epoch,
) {
    let one = builder.one();
    let current_epoch_target_diff = builder.sub(current_epoch, target_epoch);
    let target_is_valid_pred = builder.lte(current_epoch_target_diff, one);
    assert_is_true(builder, target_is_valid_pred);
}

fn validate_source_target_diff<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source_epoch: Epoch,
    target_epoch: Epoch,
) {
    let one = builder.one();
    let two = builder.constant(2);

    let diff = builder.sub(target_epoch, source_epoch);
    let diff_is_one_pred = builder.is_equal(diff, one);
    let diff_is_two_pred = builder.is_equal(diff, two);
    let diff_is_valid_pred = builder.or(diff_is_one_pred, diff_is_two_pred);
    assert_is_true(builder, diff_is_valid_pred);
}

fn is_supermajority_link<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    target_balance: Gwei,
    total_active_balance: Gwei,
) -> BoolVariable {
    let three = builder.constant::<Gwei>(3);
    let two = builder.constant::<Gwei>(2);

    let target_balance_three_times = builder.mul(target_balance, three);
    let total_active_balance_two_times = builder.mul(total_active_balance, two);
    builder.gte(target_balance_three_times, total_active_balance_two_times)
}

pub fn get_previous_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    current_epoch: Epoch,
) -> Epoch {
    let one = builder.one();
    builder.sub(current_epoch, one)
}

pub fn assert_epoch_is_not_genesis_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    epoch: Epoch,
) {
    let one = builder.one();
    let pred = builder.gte(epoch, one);
    assert_is_true(builder, pred);
}

pub fn get_current_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    let slots_per_epoch = builder.constant::<U64Variable>(SLOTS_PER_EPOCH);
    builder.div(slot, slots_per_epoch)
}

// Only sets the justification bits
pub fn process_justifications<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_active_balance: Gwei,
    previous_epoch_target_balance: Gwei,
    current_epoch_target_balance: Gwei,
    justification_bits: JustificationBitsVariable,
) -> JustificationBitsVariable {
    let prev_epoch_supermajority_pred =
        is_supermajority_link(builder, previous_epoch_target_balance, total_active_balance);

    let mut bits = justification_bits.shift_right(builder);
    bits = bits.assign_nth_bit(
        1,
        builder.or(prev_epoch_supermajority_pred, justification_bits.bits[0]),
    );
    bits.assign_nth_bit(
        0,
        is_supermajority_link(builder, current_epoch_target_balance, total_active_balance),
    )
}

pub fn validate_source<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source: CheckpointVariable,
    target_idx: U64Variable,
    current_justified_checkpoint: CheckpointVariable,
    previous_justified_checkpoint: CheckpointVariable,
) {
    let zero = builder.zero();
    let one = builder.one();

    let source_is_current_justified_checkpoint_pred =
        builder.is_equal(source.clone(), current_justified_checkpoint);
    let source_is_previous_justified_checkpoint_pred =
        builder.is_equal(source.clone(), previous_justified_checkpoint);

    let target_is_current_epoch_pred = builder.is_equal(target_idx, zero);
    let target_is_previous_epoch_pred = builder.is_equal(target_idx, one);

    let is_valid_pair_1_pred = builder.and(
        target_is_current_epoch_pred,
        source_is_current_justified_checkpoint_pred,
    );
    let is_valid_pair_2_pred = builder.and(
        target_is_previous_epoch_pred,
        source_is_previous_justified_checkpoint_pred,
    );
    let is_valid_pair_pred = builder.or(is_valid_pair_1_pred, is_valid_pair_2_pred);
    assert_is_true(builder, is_valid_pair_pred);
}

pub fn assert_bits_are_set_in_range<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable],
    target_idx: U64Variable,
    source_idx: U64Variable,
) {
    let mut in_range: Variable = builder.zero();
    let mut bits_are_set_pred = builder._true();

    for i in 0..bits.len() {
        let idx = builder.constant(i as u64);
        let at_target_pred = builder.is_equal(idx, target_idx);
        let at_source_pred = builder.is_equal(idx, source_idx);

        in_range = builder.add(in_range, at_target_pred.variable);

        let out_of_range_pred = builder.is_zero(in_range);
        let valid_bit_pred = builder.or(bits[i], out_of_range_pred);
        bits_are_set_pred = builder.and(bits_are_set_pred, valid_bit_pred);

        in_range = builder.sub(in_range, at_source_pred.variable);
    }

    assert_is_true(builder, bits_are_set_pred);
}

pub fn assert_source_is_finalized<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    justification_bits: JustificationBitsVariable,
    previous_justified_checkpoint: CheckpointVariable,
    current_justified_checkpoint: CheckpointVariable,
    current_epoch: Epoch,
    source: CheckpointVariable,
    target: CheckpointVariable,
) {
    validate_target(builder, current_epoch, target.epoch);
    validate_source_target_diff(builder, source.epoch, target.epoch);

    let target_source_diff = builder.sub(target.epoch, source.epoch);
    let target_idx = builder.sub(current_epoch, target.epoch);
    let source_idx = builder.add(target_idx, target_source_diff);

    validate_source(
        builder,
        source,
        target_idx,
        current_justified_checkpoint,
        previous_justified_checkpoint,
    );

    assert_bits_are_set_in_range(
        builder,
        justification_bits.bits.as_slice(),
        target_idx,
        source_idx,
    );
}
