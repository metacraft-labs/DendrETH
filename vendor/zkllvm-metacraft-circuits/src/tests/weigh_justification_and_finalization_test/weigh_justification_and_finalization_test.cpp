#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "circuit_utils/base_types.h"
#include "circuit_utils/circuit_byte_utils.h"
#include "circuit_utils/constants.h"

#include <algorithm>
#include <array>

#include <llvm/ObjectYAML/YAML.h>
#include <iostream>
#include <fstream>
#include <streambuf>

#include "utils/picosha2.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"

using llvm::yaml::Input;
using llvm::yaml::IO;
using llvm::yaml::MappingTraits;
using llvm::yaml::Output;

using namespace circuit_byte_utils;
using namespace file_utils;

using std::cout;

// #################################################################################################

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

        std::array<unsigned char, 64> data {};
        size_t data_index = 0;
        for (size_t j = 0; j < 32; j++) {
            data[data_index++] = left[j];
        }
        for (size_t j = 0; j < 32; j++) {
            data[data_index++] = right[j];
        }

        picosha2::hash256(data.begin(), data.end(), hash.begin(), hash.end());
    }

    return hash;
}

template<uint32_t MERKLE_DEPTH>
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

void verify_slot(const Root& beacon_state_root, const Slot& slot, const BeaconStateLeafProof& proof) {
    auto slot_leaf = hash_tree_root(slot);
    auto gindex = BEACON_STATE_SLOT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(beacon_state_root, slot_leaf, proof, gindex);
}

void verify_previous_justified_checkpoint(const Root& beacon_state_root,
                                          const CheckpointVariable& checkpoint,
                                          const BeaconStateLeafProof& proof) {
    const auto checkpoint_leaf = hash_tree_root(checkpoint);
    const auto gindex = BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(beacon_state_root, checkpoint_leaf, proof, gindex);
}

void verify_current_justified_checkpoint(Root beacon_state_root,
                                         CheckpointVariable checkpoint,
                                         BeaconStateLeafProof proof) {
    auto checkpoint_leaf = hash_tree_root(checkpoint);
    auto gindex = BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(beacon_state_root, checkpoint_leaf, proof, gindex);
}

void verify_justification_bits(Root beacon_state_root,
                               JustificationBitsVariable justification_bits,
                               BeaconStateLeafProof proof) {
    auto justification_bits_leaf = hash_tree_root(justification_bits);
    auto gindex = BEACON_STATE_JUSTIFICATION_BITS_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(beacon_state_root, justification_bits_leaf, proof, gindex);
}

Epoch get_current_epoch(Slot slot) {
    auto slots_per_epoch = SLOTS_PER_EPOCH;
    return slot / slots_per_epoch;
}

void assert_epoch_is_not_genesis_epoch(Epoch epoch) {
    assert_true(epoch >= 1);
}

Epoch get_previous_epoch(Epoch current_epoch) {
    return current_epoch - 1;
}

Slot compute_start_slot_at_epoch_in_block_roots(Epoch epoch) {
    auto slots_per_epoch = SLOTS_PER_EPOCH;
    auto slots_per_historical_root = SLOTS_PER_HISTORICAL_ROOT;
    auto start_slot_at_epoch = epoch * slots_per_epoch;
    return start_slot_at_epoch % slots_per_historical_root;
}

void verify_epoch_start_slot_root_in_block_roots(Root beacon_state_root,
                                                 Epoch epoch,
                                                 Root block_root,
                                                 MerkleProof<18>
                                                     proof) {
    auto start_block_roots_gindex = DEPTH18_START_BLOCK_ROOTS_GINDEX;
    auto index_in_block_roots = compute_start_slot_at_epoch_in_block_roots(epoch);
    auto gindex = start_block_roots_gindex + index_in_block_roots;
    ssz_verify_proof<array_size<decltype(proof)>::size>(beacon_state_root, block_root, proof, gindex);
}

void verify_finalized_checkpoint(Root beacon_state_root,
                                 CheckpointVariable finalized_checkpoint,
                                 BeaconStateLeafProof proof) {
    auto finalized_checkpoint_leaf = hash_tree_root(finalized_checkpoint);
    auto gindex = BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(
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
    const uint64_t one = 1;
    const uint64_t two = 2;
    const uint64_t three = 3;

    const auto bits_set_1_through_4_pred = justification_bits.test_range(1, 4);
    const auto bits_set_1_through_3_pred = justification_bits.test_range(1, 3);
    const auto bits_set_0_through_3_pred = justification_bits.test_range(0, 3);
    const auto bits_set_0_through_2_pred = justification_bits.test_range(0, 2);

    const auto previous_justified_checkpoint_epoch_plus_three = previous_justified_checkpoint.epoch + three;
    const auto previous_justified_checkpoint_epoch_plus_two = previous_justified_checkpoint.epoch + two;
    const auto current_justified_checkpoint_epoch_plus_two = current_justified_checkpoint.epoch + two;
    const auto current_justified_checkpoint_epoch_plus_one = current_justified_checkpoint.epoch + one;

    const auto second_using_fourth_as_source_pred = previous_justified_checkpoint_epoch_plus_three == current_epoch;

    const auto second_using_third_as_source_pred = previous_justified_checkpoint_epoch_plus_two == current_epoch;

    const auto first_using_third_as_source_pred = current_justified_checkpoint_epoch_plus_two == current_epoch;

    const auto first_using_second_as_source_pred = current_justified_checkpoint_epoch_plus_one == current_epoch;

    const auto should_finalize_previous_justified_checkpoint_1_pred =
        bits_set_1_through_4_pred && second_using_fourth_as_source_pred;

    const auto should_finalize_previous_justified_checkpoint_2_pred =
        bits_set_1_through_3_pred && second_using_third_as_source_pred;

    const auto should_finalize_previous_justified_checkpoint_pred =
        should_finalize_previous_justified_checkpoint_1_pred || should_finalize_previous_justified_checkpoint_2_pred;

    const auto should_finalize_current_justified_checkpoint_1_pred =
        bits_set_0_through_3_pred && first_using_third_as_source_pred;

    const auto should_finalize_current_justified_checkpoint_2_pred =
        bits_set_0_through_2_pred && first_using_second_as_source_pred;

    const auto should_finalize_current_justified_checkpoint_pred =
        should_finalize_current_justified_checkpoint_1_pred || should_finalize_current_justified_checkpoint_2_pred;

    auto new_finalized_checkpoint =
        should_finalize_previous_justified_checkpoint_pred ? previous_justified_checkpoint : finalized_checkpoint;

    new_finalized_checkpoint =
        should_finalize_current_justified_checkpoint_pred ? current_justified_checkpoint : new_finalized_checkpoint;

    return new_finalized_checkpoint;
}

void weigh_justification_and_finalization(
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
    verify_slot(beacon_state_root, slot, slot_proof);

    verify_previous_justified_checkpoint(
        beacon_state_root, previous_justified_checkpoint, previous_justified_checkpoint_proof);

    verify_current_justified_checkpoint(
        beacon_state_root, current_justified_checkpoint, current_justified_checkpoint_proof);

    verify_justification_bits(beacon_state_root, justification_bits, justification_bits_proof);

    auto current_epoch = get_current_epoch(slot);
    std::cout << "current_epoch = " << current_epoch << "\n";
    assert_epoch_is_not_genesis_epoch(current_epoch);

    auto previous_epoch = get_previous_epoch(current_epoch);
    std::cout << "previous_epoch = " << previous_epoch << "\n";

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

// #################################################################################################

std::ostream& operator<<(std::ostream& os, const CheckpointVariable& c) {
    os << "CheckpointValue { epoch: " << c.epoch << ", root: " << byte_utils::bytesToHex(c.root) << " }";
    return os;
}

std::ostream& operator<<(std::ostream& os, const JustificationBitsVariable& j) {
    os << " [" << (j.bits[0] ? "true, " : "false, ") << (j.bits[1] ? "true, " : "false, ")
       << (j.bits[2] ? "true, " : "false, ") << (j.bits[2] ? "true]" : "false]");
    return os;
}

void test_circuit_sample_data() {

    auto beacon_state_root =
        byte_utils::hexToBytes<32>("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

    auto slot = 6953401;

    BeaconStateLeafProof slot_proof {
        byte_utils::hexToBytes<32>("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a"),
        byte_utils::hexToBytes<32>("96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1"),
        byte_utils::hexToBytes<32>("ef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f"),
        byte_utils::hexToBytes<32>("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        byte_utils::hexToBytes<32>("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    CheckpointVariable previous_justified_checkpoint {
        217291,
        byte_utils::hexToBytes<32>("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    BeaconStateLeafProof previous_justified_checkpoint_proof {
        byte_utils::hexToBytes<32>("0xf7b1fc5e9ef34f7455c8cc475a93eccc5cd05a3879e983a2bad46bbcbb2c71f5"),
        byte_utils::hexToBytes<32>("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        byte_utils::hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        byte_utils::hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        byte_utils::hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    CheckpointVariable current_justified_checkpoint {
        217292,
        byte_utils::hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1"),
    };

    BeaconStateLeafProof current_justified_checkpoint_proof {
        byte_utils::hexToBytes<32>("0x2b913be7c761bbb483a1321ff90ad13669cbc422c8e23eccf9eb0137c8c3cf48"),
        byte_utils::hexToBytes<32>("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        byte_utils::hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        byte_utils::hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        byte_utils::hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    JustificationBitsVariable justification_bits {true, true, true, true};

    BeaconStateLeafProof justification_bits_proof {
        byte_utils::hexToBytes<32>("0x1fca1f5d922549df42d4b5ca272bd4d022a77d520a201d5f24739b93f580a4e0"),
        byte_utils::hexToBytes<32>("0x9f1e3e59c7a4606e788c4e546a573a07c6c2e66ebd245aba2ff966b27e8c2d4f"),
        byte_utils::hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        byte_utils::hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        byte_utils::hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    MerkleProof<18> previous_epoch_start_slot_root_in_block_roots_proof {
        byte_utils::hexToBytes<32>("0x73dea1035b1bd431ccd1eaa893ad5f4b8488e68d2ca90615e5be0d8f7ba5a650"),
        byte_utils::hexToBytes<32>("0x0f7c6aa59235e573a4cdfb9411d5e4eb6255571814906c5928c016626aa2ff0a"),
        byte_utils::hexToBytes<32>("0xf770f73c2e01ddf6c71765e327eebb7bab0ab13f4506c736dfd6556037c0e646"),
        byte_utils::hexToBytes<32>("0x036f0750c86fdc21edee72b6ac1b5f728eed354c99d3b6862adf60f72bc5dcbe"),
        byte_utils::hexToBytes<32>("0x9730c8f3978ea7a1797603b19514e74273898f2be969ca8c583f55d14cd08d03"),
        byte_utils::hexToBytes<32>("0x47b601e8c14026380bdd0f716a4188e9f50a86bc58f0c342ead2a075ba8e5bc0"),
        byte_utils::hexToBytes<32>("0x6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        byte_utils::hexToBytes<32>("0x82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        byte_utils::hexToBytes<32>("0x30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        byte_utils::hexToBytes<32>("0xc9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        byte_utils::hexToBytes<32>("0x606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        byte_utils::hexToBytes<32>("0x4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        byte_utils::hexToBytes<32>("0xf3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        byte_utils::hexToBytes<32>("0xc524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        byte_utils::hexToBytes<32>("0xe3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        byte_utils::hexToBytes<32>("0x844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        byte_utils::hexToBytes<32>("0x2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        byte_utils::hexToBytes<32>("0x71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    MerkleProof<18> current_epoch_start_slot_root_in_block_roots_proof {
        byte_utils::hexToBytes<32>("c798192e5a066fe1ff3fc632bccd30a1ff47dc4d36909725db43ca6b23a5a7ba"),
        byte_utils::hexToBytes<32>("3161f17c79044792fc7c965a3fcb105f595bf895a44a774b871fa3017f5a36cc"),
        byte_utils::hexToBytes<32>("e3dddf89fa44413c3d4cf1762d7500b169116125194d96e86257cb616949560f"),
        byte_utils::hexToBytes<32>("3bfbdebbb29b9e066e08350d74f66116b0221c7d2c98724288a8e02bc7f937ae"),
        byte_utils::hexToBytes<32>("f50adbe1bff113f5d5535eee3687ac3b554af1eb56f8c966e572f8db3a61add3"),
        byte_utils::hexToBytes<32>("1a973e9b4fc1f60aea6d1453fe3418805a71fd6043f27a1c32a28bfcb67dd0eb"),
        byte_utils::hexToBytes<32>("6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        byte_utils::hexToBytes<32>("82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        byte_utils::hexToBytes<32>("30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        byte_utils::hexToBytes<32>("c9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        byte_utils::hexToBytes<32>("606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        byte_utils::hexToBytes<32>("4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        byte_utils::hexToBytes<32>("f3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        byte_utils::hexToBytes<32>("c524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        byte_utils::hexToBytes<32>("e3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        byte_utils::hexToBytes<32>("844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        byte_utils::hexToBytes<32>("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        byte_utils::hexToBytes<32>("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    auto previous_epoch_start_slot_root_in_block_roots =
        byte_utils::hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1");
    auto current_epoch_start_slot_root_in_block_roots =
        byte_utils::hexToBytes<32>("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c");

    auto total_active_balance = 10;
    auto previous_epoch_target_balance = 10;
    auto current_epoch_target_balance = 20;

    CheckpointVariable finalized_checkpoint {
        217291,
        byte_utils::hexToBytes<32>("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    BeaconStateLeafProof finalized_checkpoint_proof {
        byte_utils::hexToBytes<32>("0x26803d08d4a1a3d223ed199292fa78e62ef586391213548388375f302acfdece"),
        byte_utils::hexToBytes<32>("0xf0af1bff0357d4eb3b97bd6f7310a71acaff5c1c1d9dde7f20295b2002feccaf"),
        byte_utils::hexToBytes<32>("0x43e892858dc13eaceecec6b690cf33b7b85218aa197eb1db33de6bea3d3374c2"),
        byte_utils::hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        byte_utils::hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    CheckpointVariable new_previous_justified_checkpoint;
    CheckpointVariable new_current_justified_checkpoint;
    CheckpointVariable new_finalized_checkpoint;
    JustificationBitsVariable new_justification_bits;

    weigh_justification_and_finalization(beacon_state_root,
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
                                         new_previous_justified_checkpoint,
                                         new_current_justified_checkpoint,
                                         new_finalized_checkpoint,
                                         new_justification_bits);

    std::cout << "outputs:\n";
    std::cout << "new_previous_justified_checkpoint: " << new_previous_justified_checkpoint << "\n";
    std::cout << "new_current_justified_checkpoint: " << new_current_justified_checkpoint << "\n";
    std::cout << "new_finalized_checkpoint: " << new_finalized_checkpoint << "\n";
    std::cout << "new_justification_bits: " << new_justification_bits << "\n";

    const CheckpointVariable expected_new_previous_justified_checkpoint {
        217292, byte_utils::hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1")};
    const CheckpointVariable expected_new_current_justified_checkpoint {
        217293, byte_utils::hexToBytes<32>("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c")};
    const CheckpointVariable expected_new_finalized_checkpoint {
        217292, byte_utils::hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1")};
    const JustificationBitsVariable expected_new_justification_bits {true, true, true, true};

    std::cout << "expected outputs:\n";
    std::cout << "new_previous_justified_checkpoint: " << expected_new_previous_justified_checkpoint << "\n";
    std::cout << "new_current_justified_checkpoint: " << expected_new_current_justified_checkpoint << "\n";
    std::cout << "new_finalized_checkpoint: " << expected_new_finalized_checkpoint << "\n";
    std::cout << "new_justification_bits: " << expected_new_justification_bits << "\n";

    assert_true(expected_new_previous_justified_checkpoint == new_previous_justified_checkpoint);
    assert_true(expected_new_current_justified_checkpoint == new_current_justified_checkpoint);
    assert_true(expected_new_finalized_checkpoint == new_finalized_checkpoint);
    assert_true(expected_new_justification_bits == new_justification_bits);
}

int main(int argc, char* argv[]) {

    test_circuit_sample_data();

    return 0;
}
