use super::{checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable};
use crate::{
    constants::SLOTS_PER_EPOCH,
    types::{Epoch, Gwei, Slot},
    utils::plonky2x_extensions::assert_is_true,
};
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable};

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

pub fn process_justifications<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_active_balance: Gwei,
    previous_epoch_target_balance: Gwei,
    current_epoch_target_balance: Gwei,
    justification_bits: JustificationBitsVariable,
    current_justified_checkpoint: CheckpointVariable,
    previous_epoch_checkpoint: CheckpointVariable,
    current_epoch_checkpoint: CheckpointVariable,
) -> (CheckpointVariable, JustificationBitsVariable) {
    let previous_epoch_supermajority_link_pred =
        is_supermajority_link(builder, previous_epoch_target_balance, total_active_balance);
    let current_epoch_supermajority_link_pred =
        is_supermajority_link(builder, current_epoch_target_balance, total_active_balance);

    let mut new_current_justified_checkpoint = builder.select(
        previous_epoch_supermajority_link_pred,
        previous_epoch_checkpoint,
        current_justified_checkpoint,
    );

    new_current_justified_checkpoint = builder.select(
        current_epoch_supermajority_link_pred,
        current_epoch_checkpoint,
        new_current_justified_checkpoint,
    );

    let _true = builder._true();
    let new_second_justification_bit = builder.select(
        previous_epoch_supermajority_link_pred,
        _true,
        justification_bits.bits[0],
    );

    let mut new_justification_bits = justification_bits.shift_right(builder);
    new_justification_bits = new_justification_bits.assign_nth_bit(1, new_second_justification_bit);
    new_justification_bits =
        new_justification_bits.assign_nth_bit(0, current_epoch_supermajority_link_pred);

    (new_current_justified_checkpoint, new_justification_bits)
}

pub fn process_finalizations<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    justification_bits: JustificationBitsVariable,
    previous_justified_checkpoint: CheckpointVariable,
    current_justified_checkpoint: CheckpointVariable,
    current_epoch: Epoch,
    finalized_checkpoint: CheckpointVariable,
) -> CheckpointVariable {
    let one = builder.constant::<U64Variable>(1);
    let two = builder.constant::<U64Variable>(2);
    let three = builder.constant::<U64Variable>(3);

    let bits_set_1_through_4_pred = justification_bits.test_range(builder, 1, 4);
    let bits_set_1_through_3_pred = justification_bits.test_range(builder, 1, 3);
    let bits_set_0_through_3_pred = justification_bits.test_range(builder, 0, 3);
    let bits_set_0_through_2_pred = justification_bits.test_range(builder, 0, 2);

    let previous_justified_checkpoint_epoch_plus_three =
        builder.add(previous_justified_checkpoint.epoch, three);
    let previous_justified_checkpoint_epoch_plus_two =
        builder.add(previous_justified_checkpoint.epoch, two);
    let current_justified_checkpoint_epoch_plus_two =
        builder.add(current_justified_checkpoint.epoch, two);
    let current_justified_checkpoint_epoch_plus_one =
        builder.add(current_justified_checkpoint.epoch, one);

    let second_using_fourth_as_source_pred = builder.is_equal(
        previous_justified_checkpoint_epoch_plus_three,
        current_epoch,
    );

    let second_using_third_as_source_pred =
        builder.is_equal(previous_justified_checkpoint_epoch_plus_two, current_epoch);

    let first_using_third_as_source_pred =
        builder.is_equal(current_justified_checkpoint_epoch_plus_two, current_epoch);

    let first_using_second_as_source_pred =
        builder.is_equal(current_justified_checkpoint_epoch_plus_one, current_epoch);

    let should_finalize_previous_justified_checkpoint_1_pred = builder.and(
        bits_set_1_through_4_pred,
        second_using_fourth_as_source_pred,
    );

    let should_finalize_previous_justified_checkpoint_2_pred =
        builder.and(bits_set_1_through_3_pred, second_using_third_as_source_pred);

    let should_finalize_previous_justified_checkpoint_pred = builder.or(
        should_finalize_previous_justified_checkpoint_1_pred,
        should_finalize_previous_justified_checkpoint_2_pred,
    );

    let should_finalize_current_justified_checkpoint_1_pred =
        builder.and(bits_set_0_through_3_pred, first_using_third_as_source_pred);

    let should_finalize_current_justified_checkpoint_2_pred =
        builder.and(bits_set_0_through_2_pred, first_using_second_as_source_pred);

    let should_finalize_current_justified_checkpoint_pred = builder.or(
        should_finalize_current_justified_checkpoint_1_pred,
        should_finalize_current_justified_checkpoint_2_pred,
    );

    let mut new_finalized_checkpoint = builder.select(
        should_finalize_previous_justified_checkpoint_pred,
        previous_justified_checkpoint,
        finalized_checkpoint,
    );

    new_finalized_checkpoint = builder.select(
        should_finalize_current_justified_checkpoint_pred,
        current_justified_checkpoint,
        new_finalized_checkpoint,
    );

    new_finalized_checkpoint
}
