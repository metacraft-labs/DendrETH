use plonky2x::{
    frontend::vars::SSZVariable,
    prelude::{CircuitBuilder, CircuitVariable, PlonkParameters, U64Variable},
};

use crate::{
    checkpoint::CheckpointVariable,
    constants::{SLOTS_PER_EPOCH, SLOTS_PER_HISTORICAL_ROOT},
    justification_bits::JustificationBitsVariable,
    types::{BeaconStateLeafProof, Epoch, MerkleProof, Root, Slot},
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

pub fn verify_slot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    slot: Slot,
    proof: BeaconStateLeafProof,
) {
    let slot_leaf = slot.hash_tree_root(builder);
    let gindex = U64Variable::constant(builder, 34);
    builder.ssz_verify_proof(beacon_state_root, slot_leaf, proof.as_slice(), gindex);
}

pub fn verify_previous_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(50);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

pub fn verify_current_justified_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let checkpoint_leaf = builder.ssz_hash_tree_root(checkpoint);
    let gindex = builder.constant::<U64Variable>(51);
    builder.ssz_verify_proof(beacon_state_root, checkpoint_leaf, proof.as_slice(), gindex);
}

pub fn verify_justification_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    justification_bits: JustificationBitsVariable,
    proof: BeaconStateLeafProof,
) {
    let justification_bits_leaf = justification_bits.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(49);
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
    let first_block_roots_gindex = builder.constant::<U64Variable>(303104);
    let index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(builder, epoch);
    let gindex = builder.add(first_block_roots_gindex, index_in_block_roots);
    builder.ssz_verify_proof(beacon_state_root, block_root, proof.as_slice(), gindex);
}

pub fn verify_finalized_checkpoint<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    beacon_state_root: Root,
    finalized_checkpoint: CheckpointVariable,
    proof: BeaconStateLeafProof,
) {
    let finalized_checkpoint_leaf = finalized_checkpoint.hash_tree_root(builder);
    let gindex = builder.constant::<U64Variable>(52);
    builder.ssz_verify_proof(
        beacon_state_root,
        finalized_checkpoint_leaf,
        proof.as_slice(),
        gindex,
    );
}
