use super::{checkpoint::CheckpointVariable, justification_bits::JustificationBitsVariable};
use crate::{
    constants::{
        BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX, BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX,
        BEACON_STATE_JUSTIFICATION_BITS_GINDEX, BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX,
        BEACON_STATE_SLOT_GINDEX, DEPTH18_START_BLOCK_ROOTS_GINDEX, SLOTS_PER_EPOCH,
        SLOTS_PER_HISTORICAL_ROOT,
    },
    types::{BeaconStateLeafProof, Epoch, MerkleProof, Root, Slot},
    utils::plonky2x_extensions::assert_is_false,
};
use plonky2x::{
    frontend::vars::SSZVariable,
    prelude::{CircuitBuilder, CircuitVariable, PlonkParameters, U64Variable},
};

fn compute_start_slot_at_epoch_in_block_roots<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    epoch: Epoch,
) -> Slot {
    let slots_per_epoch = builder.constant::<U64Variable>(SLOTS_PER_EPOCH);
    let slots_per_historical_root = builder.constant::<U64Variable>(SLOTS_PER_HISTORICAL_ROOT);
    let start_slot_at_epoch = builder.mul(epoch, slots_per_epoch);
    builder.rem(start_slot_at_epoch, slots_per_historical_root)
}

pub fn assert_slot_is_not_first_in_epoch<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    slot: Slot,
) {
    let slots_per_epoch = builder.constant::<U64Variable>(SLOTS_PER_EPOCH);
    let slot_in_epoch = builder.rem(slot, slots_per_epoch);

    let zero = builder.zero();
    let slot_is_first_in_epoch_pred = builder.is_equal(slot_in_epoch, zero);
    assert_is_false(builder, slot_is_first_in_epoch_pred);
}

pub fn verify_slot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    slot: Slot,
    proof: BeaconStateLeafProof,
) {
    let slot_leaf = slot.hash_tree_root(builder);
    let gindex = U64Variable::constant(builder, BEACON_STATE_SLOT_GINDEX);
    builder.ssz_verify_proof(beacon_state_root, slot_leaf, proof.as_slice(), gindex);
}

pub fn verify_previous_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

pub fn verify_current_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

pub fn verify_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    justification_bits: JustificationBitsVariable,
    proof: BeaconStateLeafProof,
) {
    let justification_bits_leaf = justification_bits.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(BEACON_STATE_JUSTIFICATION_BITS_GINDEX);
    builder.ssz_verify_proof(
        beacon_state_root,
        justification_bits_leaf,
        proof.as_slice(),
        gindex,
    );
}

pub fn verify_epoch_start_slot_root_in_block_roots<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    epoch: Epoch,
    block_root: Root,
    proof: MerkleProof<18>,
) {
    let start_block_roots_gindex =
        builder.constant::<U64Variable>(DEPTH18_START_BLOCK_ROOTS_GINDEX);
    let index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(builder, epoch);
    let gindex = builder.add(start_block_roots_gindex, index_in_block_roots);
    builder.ssz_verify_proof(beacon_state_root, block_root, proof.as_slice(), gindex);
}

pub fn verify_finalized_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    finalized_checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let finalized_checkpoint_leaf = finalized_checkpoint.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX);
    builder.ssz_verify_proof(
        beacon_state_root,
        finalized_checkpoint_leaf,
        proof.as_slice(),
        gindex,
    );
}
