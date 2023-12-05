#pragma once

#include "circuit_byte_utils.h"

using namespace circuit_byte_utils;

namespace ssz_utils {

    template<size_t MERKLE_DEPTH>
    Bytes32 ssz_restore_merkle_root(const Bytes32& leaf,
                                    const std::array<Bytes32, MERKLE_DEPTH>& branch,
                                    uint64_t gindex,
                                    const uint64_t depth = MERKLE_DEPTH) {
        auto hash = leaf;

        for (size_t i = 0; i < depth; i++) {
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

            hash = sha256(left, right);
        }

        return hash;
    }

    template<size_t MERKLE_DEPTH>
    void ssz_verify_proof(const Bytes32& root,
                          const Bytes32& leaf,
                          const std::array<Bytes32, MERKLE_DEPTH>& branch,
                          const uint64_t gindex,
                          const uint64_t depth = MERKLE_DEPTH) {
        auto expected_root = ssz_restore_merkle_root(leaf, branch, gindex, depth);
        assert_true(root == expected_root);
    }

    Bytes32 hash_tree_root(uint64_t val) {
        auto bytes = int_to_bytes<uint64_t, 32, true>(val);
        return bytes;
    }

    Bytes32 hash_tree_root(const CheckpointVariable& checkpoint) {
        auto epoch_leaf = hash_tree_root(checkpoint.epoch);
        return sha256(epoch_leaf, checkpoint.root);
    }

    Bytes32 hash_tree_root(const JustificationBitsVariable& checkpoint) {
        Bytes32 ret_val {};
        for (auto i = 0; i < 4; i++) {
            if (checkpoint.bits[i]) {
                ret_val[0] = set_nth_bit(ret_val[0], i);
            }
        }

        return ret_val;
    }

}    // namespace ssz_utils
