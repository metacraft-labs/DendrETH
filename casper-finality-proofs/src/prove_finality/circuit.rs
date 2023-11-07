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

        let new_justification_bits = process_justifications(
            builder,
            total_number_of_validators,
            justification_bits,
            previous_epoch_attested_validators,
            current_epoch_attested_validators,
        );

        let new_justification_bits = new_justification_bits.bits.as_slice();
        let source_index = builder.sub(current_epoch, source.epoch);
        let target_index = builder.sub(current_epoch, target.epoch);
        for i in 0..4 {
            let current_index = builder.constant::<U64Variable>(i as u64);
            let in_range_source_index = builder.lte(current_index, source_index);
            let in_range_target_index = builder.gte(current_index, target_index);

            let in_range = builder.and(in_range_source_index, in_range_target_index);

            let in_range_or_justification_bits_value =
                builder.or(new_justification_bits[i], in_range);

            builder.assert_is_equal(
                new_justification_bits[i],
                in_range_or_justification_bits_value,
            );
        }
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
