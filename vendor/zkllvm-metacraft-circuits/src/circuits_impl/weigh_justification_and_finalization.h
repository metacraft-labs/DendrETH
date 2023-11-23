#pragma once

#include <array>
#include <algorithm>

#include "../utils/picosha2.h"
#include "../circuit_utils/circuit_byte_utils.h"

using namespace circuit_byte_utils;

namespace weigh_justification_and_finalization_ {

Bytes32 sha256_pair(const Bytes32& left, const Bytes32& right) {
    Bytes32 ret_val {};
    Bytes64 combined {};
    std::copy(left.begin(), left.end(), combined.begin());
    std::copy(right.begin(), right.end(), combined.begin() + 32);

    picosha2::hash256(combined.begin(), combined.end(), ret_val.begin(), ret_val.end());

    return ret_val;
}

template<uint32_t MERKLE_DEPTH>
Bytes32 ssz_restore_merkle_root(const Bytes32& leaf, const std::array<Bytes32, MERKLE_DEPTH>& branch, uint64_t gindex) {
    auto hash = leaf;

    for (size_t i = 0; i < MERKLE_DEPTH; i++) {
        Bytes32 left;
        Bytes32 right;

        if (get_nth_bit(gindex, i) % 2 == 1) {
            left = branch[i];
            right = hash;
        } else {
            right = branch[i];
            left = hash;
        }

        hash = sha256_pair(left, right);
    }

    return hash;
}

template<size_t MERKLE_DEPTH>
void ssz_verify_proof(const Bytes32& root,
                      const Bytes32& leaf,
                      const std::array<Bytes32, MERKLE_DEPTH>& branch,
                      const uint64_t gindex) {
    auto expected_root = ssz_restore_merkle_root<MERKLE_DEPTH>(leaf, branch, gindex);
    assert_true(root == expected_root);
}

Bytes32 hash_tree_root(uint64_t val) {
    auto bytes = int_to_bytes(val);
    Bytes32 return_val {};
    std::copy(bytes.begin(), bytes.end(), return_val.begin());
    return return_val;
}

Bytes32 hash_tree_root(const CheckpointVariable& checkpoint) {
    auto epoch_leaf = hash_tree_root(checkpoint.epoch);
    return sha256_pair(epoch_leaf, checkpoint.root);
}

Bytes32 hash_tree_root(const JustificationBitsVariable& checkpoint) {
    Bytes32 ret_val {};
    for (auto i = 0; i < 4; i++) {
        if (checkpoint.bits[i]) {
            set_nth_bit(ret_val[0], i);
        }
    }

    return ret_val;
}

void assert_epoch_is_not_genesis_epoch(Epoch epoch) {
    assert_true(epoch >= 1);
}

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
    auto start_block_roots_gindex = DEPTH18_START_BLOCK_ROOTS_GINDEX;
    auto index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(epoch);
    auto gindex = start_block_roots_gindex + index_in_block_roots;
    ssz_verify_proof(beacon_state_root, block_root, proof, gindex);
}

void verify_finalized_checkpoint(Root beacon_state_root,
                                 CheckpointVariable finalized_checkpoint,
                                 BeaconStateLeafProof proof) {
    auto finalized_checkpoint_leaf = hash_tree_root(finalized_checkpoint);
    auto gindex = BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX;
    ssz_verify_proof(
        beacon_state_root, finalized_checkpoint_leaf, proof, gindex);
}

void assert_slot_is_not_first_in_epoch(Slot slot) {
    auto slot_in_epoch = slot % SLOTS_PER_EPOCH;
    assert_true(slot_in_epoch != 0);
}

bool is_supermajority_link(Gwei target_balance, Gwei total_active_balance) {
    return target_balance * 3 >= total_active_balance * 2;
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
    const auto previous_epoch_supermajority_link_pred =
        is_supermajority_link(previous_epoch_target_balance, total_active_balance);
    const auto current_epoch_supermajority_link_pred =
        is_supermajority_link(current_epoch_target_balance, total_active_balance);

    auto new_current_justified_checkpoint =
        previous_epoch_supermajority_link_pred ? previous_epoch_checkpoint : current_justified_checkpoint;

    new_current_justified_checkpoint =
        current_epoch_supermajority_link_pred ? current_epoch_checkpoint : new_current_justified_checkpoint;

    const auto new_second_justification_bit =
        previous_epoch_supermajority_link_pred ? true : justification_bits.bits[0];

    auto new_justification_bits = justification_bits;
    new_justification_bits.shift_right(1);
    new_justification_bits.bits[1] = new_second_justification_bit;
    new_justification_bits.bits[0] = current_epoch_supermajority_link_pred;

    out_checkpoint = new_current_justified_checkpoint;
    out_justification_bits = new_justification_bits;
}

CheckpointVariable process_finalizations(const JustificationBitsVariable& justification_bits,
                                         const CheckpointVariable& previous_justified_checkpoint,
                                         const CheckpointVariable& current_justified_checkpoint,
                                         const Epoch& current_epoch,
                                         const CheckpointVariable& finalized_checkpoint) {

    auto new_finalized_checkpoint =
        ((justification_bits.test_range(1, 4) && (previous_justified_checkpoint.epoch + 3 == current_epoch)) ||
         (justification_bits.test_range(1, 3) && (previous_justified_checkpoint.epoch + 2 == current_epoch))) 
         ? previous_justified_checkpoint : finalized_checkpoint;

    new_finalized_checkpoint =
        ((justification_bits.test_range(0, 3) && (current_justified_checkpoint.epoch + 2 == current_epoch)) ||
         (justification_bits.test_range(0, 2) && (current_justified_checkpoint.epoch + 1 == current_epoch)))
         ? current_justified_checkpoint : new_finalized_checkpoint;

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
    assert_slot_is_not_first_in_epoch(slot);
    ssz_verify_proof(beacon_state_root, hash_tree_root(slot), slot_proof, BEACON_STATE_SLOT_GINDEX);

    ssz_verify_proof(beacon_state_root, hash_tree_root(previous_justified_checkpoint), previous_justified_checkpoint_proof, BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX);

    ssz_verify_proof(beacon_state_root, hash_tree_root(current_justified_checkpoint), current_justified_checkpoint_proof, BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX);

    ssz_verify_proof(beacon_state_root, hash_tree_root(justification_bits), justification_bits_proof, BEACON_STATE_JUSTIFICATION_BITS_GINDEX);

    auto current_epoch = get_current_epoch(slot);
    assert_epoch_is_not_genesis_epoch(current_epoch);

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

    CheckpointVariable new_current_justified_checkpoint {};
    JustificationBitsVariable new_justification_bits {};

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

}
