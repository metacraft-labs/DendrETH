#include "../circuits_imp/weigh_justification_and_finalization.h"

using namespace weigh_justification_and_finalization_;

struct weigh_justification_and_finalization_result {
    CheckpointVariable out_current_justified_checkpoint;
    CheckpointVariable out_new_current_justified_checkpoint;
    CheckpointVariable out_new_finalized_checkpoint;
    JustificationBitsVariable out_new_justification_bits;
};

[[circuit]] weigh_justification_and_finalization_result
    weigh_justification_and_finalization(
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
    JustificationBitsVariable& out_new_justification_bits
)
{
    weigh_justification_and_finalization_result result;
    weigh_justification_and_finalization_imp(
        beacon_state_root,
        slot,
        slot_proof,
        previous_justified_checkpoint,
        previous_justified_checkpoint_proof,
        current_justified_checkpoint,
        current_justified_checkpoint_proof,
        justification_bits,
        justification_bits_proof,
        total_active_balance,
        previous_epoch_target_balance,
        current_epoch_target_balance,
        previous_epoch_start_slot_root_in_block_roots,
        previous_epoch_start_slot_root_in_block_roots_proof,
        current_epoch_start_slot_root_in_block_roots,
        current_epoch_start_slot_root_in_block_roots_proof,
        finalized_checkpoint,
        finalized_checkpoint_proof,
        // Outputs:
        result.out_current_justified_checkpoint,    // new previous justified checkpoint
        result.out_new_current_justified_checkpoint,
        result.out_new_finalized_checkpoint,
        result.out_new_justification_bits);
    return result;
}
