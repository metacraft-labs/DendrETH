use crate::{
    types::Gwei,
    utils::plonky2x_extensions::assert_is_true,
    weigh_justification_and_finalization::{
        checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable
    },
};
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable},
};

#[derive(Debug, Clone)]
pub struct ProveFinality;

impl Circuit for ProveFinality {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let source = builder.read::<CheckpointVariable>();
        let target = builder.read::<CheckpointVariable>();
        let slot = builder.read::<Gwei>();
        let total_number_of_validators = builder.read::<Gwei>();
        let justification_bits = builder.read::<JustificationBitsVariable>();
        let previous_epoch_attested_validators = builder.read::<Gwei>();
        let current_epoch_attested_validators = builder.read::<Gwei>();
        let previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let current_justified_checkpoint = builder.read::<CheckpointVariable>();

        validate_target_source_difference(builder, source.clone(), target.clone());

        let new_justification_bits = process_justifications(
            builder,
            total_number_of_validators,
            justification_bits,
            previous_epoch_attested_validators,
            current_epoch_attested_validators,
        );

        let thirty_two = builder.constant::<U64Variable>(32);
        let new_justification_bits = new_justification_bits.bits.as_slice();
        let current_epoch = builder.div(slot, thirty_two);
        let source_index = builder.sub(current_epoch, source.epoch);
        let target_index = builder.sub(current_epoch, target.epoch);

        validate_source(
            builder,
            source,
            target_index,
            previous_justified_checkpoint,
            current_justified_checkpoint,
        );

        validate_justification_bits(builder, source_index, target_index, new_justification_bits);
    }
}

fn process_justifications<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    total_number_of_validators: Gwei,
    justification_bits: JustificationBitsVariable,
    previous_epoch_attested_validators: Gwei,
    current_epoch_attested_validators: Gwei,
) -> JustificationBitsVariable {
    let previous_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        total_number_of_validators,
        previous_epoch_attested_validators,
    );

    let current_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        builder,
        total_number_of_validators,
        current_epoch_attested_validators,
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

    new_justification_bits
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

pub fn validate_target_source_difference<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source: CheckpointVariable,
    target: CheckpointVariable,
) {
    let one = builder.one();
    let two = builder.constant::<U64Variable>(2);

    let target_source_difference = builder.sub(target.epoch, source.epoch);
    let target_source_difference_equals_one = builder.is_equal(target_source_difference, one);
    let target_source_difference_equals_two = builder.is_equal(target_source_difference, two);
    let target_source_difference_equals_one_or_two = builder.or(
        target_source_difference_equals_one,
        target_source_difference_equals_two,
    );
    assert_is_true(builder, target_source_difference_equals_one_or_two);
}

pub fn validate_source<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source: CheckpointVariable,
    target_idx: U64Variable,
    previous_justified_checkpoint: CheckpointVariable,
    current_justified_checkpoint: CheckpointVariable,
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

pub fn validate_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source_index_epoch: U64Variable,
    target_index_epoch: U64Variable,
    justification_bits: &[BoolVariable],
) {
    for i in 0..4 {
        let current_index = builder.constant::<U64Variable>(i as u64);
        let in_range_source_index = builder.lte(current_index, source_index_epoch);
        let in_range_target_index = builder.gte(current_index, target_index_epoch);

        let in_range = builder.and(in_range_source_index, in_range_target_index);

        let in_range_or_justification_bits_value = builder.or(justification_bits[i], in_range);

        builder.assert_is_equal(justification_bits[i], in_range_or_justification_bits_value);
    }
}
