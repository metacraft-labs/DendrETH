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
    return bitmask_attested_validators * 5 >= total_num_validators * 4;
}

void process_justifications(
    const Gwei total_number_of_validators,
    const Gwei previous_epoch_attested_validators,
    const Gwei current_epoch_attested_validators,
    const JustificationBitsVariable& justification_bits,
    const CheckpointVariable& current_justified_checkpoint,
    const CheckpointVariable& previous_epoch_checkpoint,
    const CheckpointVariable& current_epoch_checkpoint,
    // Outputs:
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
        current_epoch_attested_validators,
    );

    auto new_current_justified_checkpoint =
        previous_epoch_supermajority_link_pred ?
        previous_epoch_checkpoint : current_justified_checkpoint;

    new_current_justified_checkpoint =
        current_epoch_supermajority_link_pred ?
        current_epoch_checkpoint : new_current_justified_checkpoint;

    const auto new_second_justification_bit =
        previous_epoch_supermajority_link_pred ?
        true, justification_bits.bits[0];

    JustificationBitsVariable new_justification_bits = justification_bits;
    new_justification_bits.shift_right(1);
    new_justification_bits.bits[1] = new_second_justification_bit;
    new_justification_bits.bits[0] = current_epoch_supermajority_link_pred;

}

void prove_finality(
    const Gwei total_number_of_validators,
    const JustificationBitsVariable& justification_bits,
    const CheckpointVariable& current_justified_checkpoint,
    const CheckpointVariable& source,
    const CheckpointVariable& target,
    const Gwei previous_epoch_attested_validators,
    const Gwei current_epoch_attested_validators,
    // Outputs:
    CheckpointVariable& finalized_cp
)
{
    CheckpointVariable new_current_justified_checkpoint;
    JustificationBitsVariable new_justification_bits;

    process_justifications(
        total_number_of_validators,
        previous_epoch_attested_validators,
        current_epoch_attested_validators,
        justification_bits,
        current_justified_checkpoint,
        source,
        target,
        new_current_justified_checkpoint,
        new_justification_bits
    );

    auto target_source_difference = target.epoch - source.epoch;
    auto mapped_source = one_u64 + target_source_difference;

    Epoch accumulator = 0;
    for (int i = 1; i < 4; i++) {
        if (i <= mapped_source && new_justification_bits[i]) {
            accumulator += 1;
        }
    }

    assert_is_equal(accumulator, mapped_source);

    finalized_cp = source;

}
