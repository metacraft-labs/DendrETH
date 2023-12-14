use crate::{
    types::{Epoch, Gwei, Slot},
    utils::plonky2x_extensions::assert_is_true,
    weigh_justification_and_finalization::{
        checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable,
    },
};
use plonky2x::{
    backend::circuit::Circuit,
    frontend::vars::{ByteVariable, Bytes32Variable, BytesVariable},
    prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable},
    utils,
};

#[derive(Debug, Clone)]
pub struct ProveFinality;

impl Circuit for ProveFinality {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        utils::setup_logger();
        let source = builder.read::<CheckpointVariable>();
        let target = builder.read::<CheckpointVariable>();
        let target_source_difference = builder.sub(target.epoch, source.epoch);

        let one = builder.constant::<U64Variable>(1);
        let two = builder.constant::<U64Variable>(2);
        let target_source_difference_equals_one = builder.is_equal(target_source_difference, one);
        let target_source_difference_equals_two = builder.is_equal(target_source_difference, two);
        let target_source_difference_equals_one_or_two = builder.or(
            target_source_difference_equals_one,
            target_source_difference_equals_two,
        );
        assert_is_true(builder, target_source_difference_equals_one_or_two);

        let slot = builder.read::<Gwei>();
        let zero = builder.constant::<U64Variable>(0);
        let thirty_two = builder.constant::<U64Variable>(32);

        let current_epoch = builder.div(slot, thirty_two);
        let current_epoch_target_difference = builder.sub(current_epoch, target.epoch);
        let target_is_first_bit = builder.is_equal(current_epoch_target_difference, zero);
        let target_is_second_bit = builder.is_equal(current_epoch_target_difference, one);
        let is_target_first_or_second_bit = builder.or(target_is_first_bit, target_is_second_bit);
        assert_is_true(builder, is_target_first_or_second_bit);

        let total_number_of_validators = builder.read::<Gwei>();
        let justification_bits = builder.read::<JustificationBitsVariable>();
        let previous_epoch_attested_validators = builder.read::<Gwei>();
        let current_epoch_attested_validators = builder.read::<Gwei>();

        let previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let current_justified_checkpoint = builder.read::<CheckpointVariable>();
        let finalized_checkpoint = builder.read::<CheckpointVariable>();

        let current_epoch = get_current_epoch(builder, slot);

        let previous_epoch = get_previous_epoch(builder, current_epoch);

        let zero_byte = ByteVariable([builder._false(); 8]);
        let zero_bytes32 =
            Bytes32Variable(BytesVariable::<32>(vec![zero_byte; 32].try_into().unwrap()));
        let previous_epoch_root = zero_bytes32;

        let current_epoch_root = zero_bytes32;

        let (_, new_justification_bits) = process_justifications(
            builder,
            total_number_of_validators,
            previous_epoch_attested_validators,
            current_epoch_attested_validators,
            justification_bits,
            current_justified_checkpoint.clone(),
            CheckpointVariable {
                epoch: previous_epoch,
                root: previous_epoch_root,
            },
            CheckpointVariable {
                epoch: current_epoch,
                root: current_epoch_root,
            },
        );

        let new_finalized_checkpoint = process_finalizations(
            builder,
            new_justification_bits,
            previous_justified_checkpoint,
            current_justified_checkpoint.clone(),
            current_epoch,
            finalized_checkpoint,
        );

        builder.write::<CheckpointVariable>(target.clone());
        builder.write::<CheckpointVariable>(source.clone());
        builder.assert_is_equal(new_finalized_checkpoint.epoch, source.epoch);
    }
}

pub fn get_previous_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    current_epoch: Epoch,
) -> Epoch {
    let one = builder.one();
    builder.sub(current_epoch, one)
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
    let previous_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        total_active_balance,
        previous_epoch_target_balance,
    );
    let current_epoch_supermajority_link_pred =
        is_supermajority_link_in_votes(builder, total_active_balance, current_epoch_target_balance);

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

pub fn get_current_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) -> Epoch {
    let slots_per_epoch = builder.constant::<U64Variable>(32);
    builder.div(slot, slots_per_epoch)
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

fn is_supermajority_link_in_votes<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_num_validators: Gwei,
    bitmask_attested_validators: Gwei,
) -> BoolVariable {
    let five = builder.constant::<Gwei>(5);
    let four = builder.constant::<Gwei>(4);

    let bitmask_attested_validators_five_times = builder.mul(bitmask_attested_validators, five);
    let total_num_validators_four_times = builder.mul(total_num_validators, four);
    builder.gte(
        bitmask_attested_validators_five_times,
        total_num_validators_four_times,
    )
}
