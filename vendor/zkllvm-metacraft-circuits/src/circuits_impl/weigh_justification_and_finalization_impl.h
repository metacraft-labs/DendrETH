#pragma once

#include <array>
#include <algorithm>

#include "../utils/picosha2.h"
#include "../circuit_utils/circuit_byte_utils.h"
#include "../circuit_utils/ssz_utils.h"

using namespace circuit_byte_utils;
using namespace ssz_utils;

namespace weigh_justification_and_finalization_ {

    Epoch get_previous_epoch(Epoch current_epoch) {
        return current_epoch - 1;
    }

    Slot compute_start_slot_at_epoch_in_block_roots(Epoch epoch) {
        return (epoch * SLOTS_PER_EPOCH) % SLOTS_PER_HISTORICAL_ROOT;
    }

    void verify_epoch_start_slot_root_in_block_roots(Root beacon_state_root,
                                                     Epoch epoch,
                                                     Root block_root,
                                                     MerkleProof<18>
                                                         proof) {
        auto index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(epoch);
        auto gindex = DEPTH18_START_BLOCK_ROOTS_GINDEX + index_in_block_roots;
        ssz_verify_proof(beacon_state_root, block_root, proof, gindex);
    }

    void inline verify_finalized_checkpoint(Root beacon_state_root,
                                            CheckpointVariable finalized_checkpoint,
                                            BeaconStateLeafProof proof) {
        ssz_verify_proof(
            beacon_state_root, hash_tree_root(finalized_checkpoint), proof, BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX);
    }

    void process_justifications(const Gwei& total_active_balance,
                                const Gwei& previous_epoch_target_balance,
                                const Gwei& current_epoch_target_balance,
                                const JustificationBitsVariable& justification_bits,
                                const CheckpointVariable& current_justified_checkpoint,
                                const CheckpointVariable& previous_epoch_checkpoint,
                                const CheckpointVariable& current_epoch_checkpoint,
                                // Outputs:
                                CheckpointVariable& out_checkpoint,
                                JustificationBitsVariable& out_justification_bits) {
        out_justification_bits = justification_bits;
        out_justification_bits.shift_right(1);
        out_justification_bits.bits[0] = false;
        if (previous_epoch_target_balance * 3 >= total_active_balance * 2) {
            out_checkpoint = previous_epoch_checkpoint;
            out_justification_bits.bits[1] = true;
        }
        if (current_epoch_target_balance * 3 >= total_active_balance * 2) {
            out_checkpoint = current_epoch_checkpoint;
            out_justification_bits.bits[0] = true;
        }
    }

    CheckpointVariable process_finalizations(const JustificationBitsVariable& justification_bits,
                                             const CheckpointVariable& previous_justified_checkpoint,
                                             const CheckpointVariable& current_justified_checkpoint,
                                             const Epoch& current_epoch,
                                             const CheckpointVariable& finalized_checkpoint) {

        CheckpointVariable new_finalized_checkpoint;
        // The 2nd/3rd/4th most recent epochs are justified, the 2nd using the 4th as source
        if (justification_bits.test_range(1, 4) && previous_justified_checkpoint.epoch + 3 == current_epoch)
            new_finalized_checkpoint = previous_justified_checkpoint;
        // The 2nd/3rd most recent epochs are justified, the 2nd using the 3rd as source
        if (justification_bits.test_range(1, 3) && previous_justified_checkpoint.epoch + 2 == current_epoch)
            new_finalized_checkpoint = previous_justified_checkpoint;
        // The 1st/2nd/3rd most recent epochs are justified, the 1st using the 3rd as source
        if (justification_bits.test_range(0, 3) && current_justified_checkpoint.epoch + 2 == current_epoch)
            new_finalized_checkpoint = current_justified_checkpoint;
        // The 1st/2nd most recent epochs are justified, the 1st using the 2nd as source
        if (justification_bits.test_range(0, 2) && current_justified_checkpoint.epoch + 1 == current_epoch)
            new_finalized_checkpoint = current_justified_checkpoint;

        return new_finalized_checkpoint;
    }

    void weigh_justification_and_finalization_impl(
        const Root& beacon_state_root,
        const Slot& slot,
        const BeaconStateLeafProof& slot_proof,
        const CheckpointVariable& previous_justified_checkpoint,
        const BeaconStateLeafProof& previous_justified_checkpoint_proof,
        const CheckpointVariable& current_justified_checkpoint,
        const BeaconStateLeafProof& current_justified_checkpoint_proof,
        const JustificationBitsVariable& justification_bits,
        const BeaconStateLeafProof& justification_bits_proof,
        const Gwei& total_active_balance,
        const Gwei& previous_epoch_target_balance,
        const Gwei& current_epoch_target_balance,
        const Root& previous_epoch_start_slot_root_in_block_roots,
        const MerkleProof<18>& previous_epoch_start_slot_root_in_block_roots_proof,
        const Root& current_epoch_start_slot_root_in_block_roots,
        const MerkleProof<18>& current_epoch_start_slot_root_in_block_roots_proof,
        const CheckpointVariable& finalized_checkpoint,
        const BeaconStateLeafProof& finalized_checkpoint_proof,
        // Outputs:
        CheckpointVariable& out_current_justified_checkpoint,    // new previous justified checkpoint
        CheckpointVariable& out_new_current_justified_checkpoint,
        CheckpointVariable& out_new_finalized_checkpoint,
        JustificationBitsVariable& out_new_justification_bits) {
        assert_true((slot % SLOTS_PER_EPOCH) != 0);
        ssz_verify_proof(beacon_state_root, hash_tree_root(slot), slot_proof, BEACON_STATE_SLOT_GINDEX);

        ssz_verify_proof(beacon_state_root,
                         hash_tree_root(previous_justified_checkpoint),
                         previous_justified_checkpoint_proof,
                         BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX);

        ssz_verify_proof(beacon_state_root,
                         hash_tree_root(current_justified_checkpoint),
                         current_justified_checkpoint_proof,
                         BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX);

        ssz_verify_proof(beacon_state_root,
                         hash_tree_root(justification_bits),
                         justification_bits_proof,
                         BEACON_STATE_JUSTIFICATION_BITS_GINDEX);

        auto current_epoch = get_current_epoch(slot);
        assert_true(current_epoch >= 1);

        auto previous_epoch = get_previous_epoch(current_epoch);

        verify_epoch_start_slot_root_in_block_roots(beacon_state_root,
                                                    previous_epoch,
                                                    previous_epoch_start_slot_root_in_block_roots,
                                                    previous_epoch_start_slot_root_in_block_roots_proof);

        verify_epoch_start_slot_root_in_block_roots(beacon_state_root,
                                                    current_epoch,
                                                    current_epoch_start_slot_root_in_block_roots,
                                                    current_epoch_start_slot_root_in_block_roots_proof);

        verify_finalized_checkpoint(beacon_state_root, finalized_checkpoint, finalized_checkpoint_proof);

        CheckpointVariable new_current_justified_checkpoint;
        JustificationBitsVariable new_justification_bits;

        process_justifications(total_active_balance,
                               previous_epoch_target_balance,
                               current_epoch_target_balance,
                               justification_bits,
                               current_justified_checkpoint,
                               CheckpointVariable {previous_epoch, previous_epoch_start_slot_root_in_block_roots},
                               CheckpointVariable {current_epoch, current_epoch_start_slot_root_in_block_roots},
                               new_current_justified_checkpoint,
                               new_justification_bits);

        auto new_finalized_checkpoint = process_finalizations(new_justification_bits,
                                                              previous_justified_checkpoint,
                                                              current_justified_checkpoint,
                                                              current_epoch,
                                                              finalized_checkpoint);

        out_current_justified_checkpoint = current_justified_checkpoint;
        out_new_current_justified_checkpoint = new_current_justified_checkpoint;
        out_new_finalized_checkpoint = new_finalized_checkpoint;
        out_new_justification_bits = new_justification_bits;
    }

}    // namespace weigh_justification_and_finalization_
