use super::{
    beacon_state_field_verification::*, checkpoint::CheckpointVariable, epoch_processing::*,
    justification_bits::JustificationBitsVariable,
};
use crate::{
    types::{BeaconStateLeafProof, Epoch, Gwei, MerkleProof, Root, Slot},
    utils::plonky2x_extensions::{are_not_equal, assert_is_not_equal, assert_is_true},
};
use plonky2x::{
    backend::circuit::{Circuit, CircuitBuild},
    frontend::uint::uint64::U64Variable,
    prelude::{CircuitBuilder, PlonkParameters},
};

#[derive(Debug, Clone)]
pub struct ProveFinality;

impl Circuit for ProveFinality {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let beacon_state_root = builder.read::<Root>();
        let slot = builder.read::<Slot>();
        let slot_proof = builder.read::<BeaconStateLeafProof>();
        let previous_justified_checkpoint = builder.read::<CheckpointVariable>();
        let previous_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        let current_justified_checkpoint = builder.read::<CheckpointVariable>();
        let current_justified_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        let justification_bits = builder.read::<JustificationBitsVariable>();
        let justification_bits_proof = builder.read::<BeaconStateLeafProof>();
        let total_active_balance = builder.read::<Gwei>();
        // TODO: Maybe this should come with a plonky2 proof which should be verified (the get_total_active_balance circuit which we are not yet sure how will work)
        let previous_epoch_target_balance = builder.read::<Gwei>();
        // TODO: Maybe this should come with a plonky2 proof which should be verified (the get_total_active_balance circuit which we are not yet sure how will work)
        let current_epoch_target_balance = builder.read::<Gwei>();
        // TODO: Maybe this should come with a plonky2 proof which should be verified (the get_total_active_balance circuit which we are not yet sure how will work)
        let previous_epoch_start_slot_root_in_block_roots = builder.read::<Root>();
        let previous_epoch_start_slot_root_in_block_roots_proof = builder.read::<MerkleProof<18>>();
        let current_epoch_start_slot_root_in_block_roots = builder.read::<Root>();
        let current_epoch_start_slot_root_in_block_roots_proof = builder.read::<MerkleProof<18>>();
        let finalized_checkpoint = builder.read::<CheckpointVariable>();
        let finalized_checkpoint_proof = builder.read::<BeaconStateLeafProof>();
        let source = builder.read::<CheckpointVariable>();
        let target = builder.read::<CheckpointVariable>();

        let current_epoch = get_current_epoch(builder, slot);

        assert_slot_is_not_first_in_epoch(builder, slot);
        verify_slot(builder, beacon_state_root, slot, slot_proof);

        verify_previous_justified_checkpoint(
            builder,
            beacon_state_root,
            previous_justified_checkpoint.clone(),
            previous_justified_checkpoint_proof,
        );

        verify_current_justified_checkpoint(
            builder,
            beacon_state_root,
            current_justified_checkpoint.clone(),
            current_justified_checkpoint_proof,
        );

        verify_justification_bits(
            builder,
            beacon_state_root,
            justification_bits.clone(),
            justification_bits_proof,
        );

        assert_epoch_is_not_genesis_epoch(builder, current_epoch);

        let previous_epoch = get_previous_epoch(builder, current_epoch);

        verify_epoch_start_slot_root_in_block_roots(
            builder,
            beacon_state_root,
            previous_epoch,
            previous_epoch_start_slot_root_in_block_roots,
            previous_epoch_start_slot_root_in_block_roots_proof,
        );

        verify_epoch_start_slot_root_in_block_roots(
            builder,
            beacon_state_root,
            current_epoch,
            current_epoch_start_slot_root_in_block_roots,
            current_epoch_start_slot_root_in_block_roots_proof,
        );

        verify_finalized_checkpoint(
            builder,
            beacon_state_root,
            finalized_checkpoint.clone(),
            finalized_checkpoint_proof,
        );

        let new_justification_bits = process_justifications(
            builder,
            total_active_balance,
            previous_epoch_target_balance,
            current_epoch_target_balance,
            justification_bits.clone(),
        );

        assert_source_is_finalized(
            builder,
            new_justification_bits.clone(),
            previous_justified_checkpoint,
            current_justified_checkpoint.clone(),
            current_epoch,
            source,
            target,
        );

        // builder.write::<CheckpointVariable>(current_justified_checkpoint); // new previous justified checkpoint
        // builder.write::<CheckpointVariable>(new_current_justified_checkpoint);
        // builder.write::<CheckpointVariable>(new_finalized_checkpoint);
        // builder.write::<JustificationBitsVariable>(new_justification_bits);
    }
}
