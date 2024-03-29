#pragma once

#include "circuit_byte_utils.h"
#include "base_types.h"

using namespace circuit_byte_utils;

namespace ssz_utils {

    template<size_t MerkleDepth, bool Full>
    HashType ssz_restore_merkle_root(const HashType& leaf,
                                     const static_vector<HashType, MerkleDepth, Full>& branch,
                                     uint64_t gindex) {
        auto hash = leaf;

        for (size_t i = 0; i < MerkleDepth; i++) {
            HashType left;
            HashType right;

            if (gindex % 2 == 1) {
                left = branch[i];
                right = hash;
            } else {
                right = branch[i];
                left = hash;
            }

            gindex /= 2;

            hash = sha256_pair(left, right);
        }

        return hash;
    }

    template<size_t MerkleDepth, bool Full>
    void ssz_verify_proof(const HashType& root,
                          const HashType& leaf,
                          const static_vector<HashType, MerkleDepth, Full>& branch,
                          const uint64_t gindex) {
        auto expected_root = ssz_restore_merkle_root(leaf, branch, gindex);
        assert_true(sha256_equals(root, expected_root));
    }

    HashType hash_tree_root(uint64_t val) {
        auto bytes = int_to_bytes<uint64_t, 32>(val);
        return bytes_to_hash_type(bytes);
    }

    HashType hash_tree_root(const CheckpointVariable& checkpoint) {
        auto epoch_leaf = hash_tree_root(checkpoint.epoch);
        return sha256_pair(epoch_leaf, checkpoint.root);
    }

    HashType hash_tree_root(const JustificationBitsVariable& checkpoint) {

        Bytes32 bytes {};
        for (auto i = 0; i < 4; i++) {
            if (checkpoint.bits[i]) {
                bytes[0] = set_nth_bit(bytes[0], i);
            }
        }
        return bytes_to_hash_type(bytes);
    }

}    // namespace ssz_utils
