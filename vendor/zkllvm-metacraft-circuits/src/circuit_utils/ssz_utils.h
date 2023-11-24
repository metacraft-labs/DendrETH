#pragma once

#include "circuit_byte_utils.h"

using namespace circuit_byte_utils;

namespace ssz_utils {

template<uint32_t MERKLE_DEPTH>
Bytes32 ssz_restore_merkle_root(const Bytes32& leaf, const std::array<Bytes32, MERKLE_DEPTH>& branch, uint64_t gindex) {
    auto hash = leaf;

    for (size_t i = 0; i < MERKLE_DEPTH; i++) {
        Bytes32 left;
        Bytes32 right;

        if (gindex % 2 == 1) {
            left = branch[i];
            right = hash;
        } else {
            right = branch[i];
            left = hash;
        }

        gindex /= 2;

        hash = calc_hash(left, right);
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
    return calc_hash(epoch_leaf, checkpoint.root);
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

}