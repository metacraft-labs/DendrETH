#pragma once

#include <array>
#include <algorithm>

#include "utils/picosha2.h"
#include "circuit_utils/circuit_byte_utils.h"

bool is_supermajority_link_in_votes(
    Gwei total_num_validators,
    Gwei bitmask_attested_validators
)
{
    auto bitmask_attested_validators_five_times = bitmask_attested_validators * 5;
    auto total_num_validators_four_times = total_num_validators * 4;
    return bitmask_attested_validators_five_times >= total_num_validators_four_times;
}

void process_votes(
    const Gwei& total_number_of_validators,
    const Gwei& previous_epoch_attested_validators,
    const Gwei& current_epoch_attested_validators,
    const JustificationBitsVariable& justification_bits,
    const CheckpointVariable& current_justified_checkpoint,
    const CheckpointVariable previous_epoch_checkpoint,
    const CheckpointVariable current_epoch_checkpoint,
    //Outputs:
    CheckpointVariable& new_current_justified_checkpoint,
    JustificationBitsVariable& new_justification_bits
) 
{
    const auto previous_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        total_number_of_validators,
        previous_epoch_attested_validators
    );

    const auto current_epoch_supermajority_link_pred = is_supermajority_link_in_votes(
        total_number_of_validators,
        current_epoch_attested_validators
    );

    new_current_justified_checkpoint = previous_epoch_supermajority_link_pred ? 
        previous_epoch_checkpoint : current_justified_checkpoint;

    new_current_justified_checkpoint = current_epoch_supermajority_link_pred ?
        current_epoch_checkpoint : new_current_justified_checkpoint;

    const auto new_second_justification_bit = previous_epoch_supermajority_link_pred ?
        true : justification_bits.bits[0];

    new_justification_bits = justification_bits;
    new_justification_bits.shift_right(1);
    new_justification_bits.bits[1] = new_second_justification_bit;
    new_justification_bits.bits[0] = current_epoch_supermajority_link_pred;

}

void prove_finality(
    Gwei total_number_of_validators,
    Slot slot,
    Gwei _bitmask_attested_validators,
    JustificationBitsVariable justification_bits,
    CheckpointVariable current_justified_checkpoint,
    CheckpointVariable source,
    CheckpointVariable target,
    Gwei previous_epoch_attested_validators,
    Gwei current_epoch_attested_validators
)
{
    //let zero_bit = builder.constant::<BoolVariable>(false);
    auto current_epoch = get_current_epoch(slot);
    auto _previous_epoch = get_previous_epoch(current_epoch);
    AttestedValidators _bitmask;

    CheckpointVariable new_current_justified_checkpoint;
    JustificationBitsVariable new_justification_bits;

    process_votes(
        total_number_of_validators,
        previous_epoch_attested_validators,
        current_epoch_attested_validators,
        justification_bits,
        current_justified_checkpoint,
        source,
        target,
        //Outputs:
        new_current_justified_checkpoint,
        new_justification_bits
    );

    // iterate trough source & target and check if the bits are ones
    auto target_source_difference = target.epoch - source.epoch;
    auto mapped_source = target_source_difference + 1;

    auto new_finalized_checkpoint = CheckpointVariable {
        mapped_source,
        source.root,
    };
}
