#pragma once

#include "circuit_byte_utils.h"
#include "base_types.h"

using namespace circuit_byte_utils;

namespace ssz_utils {

    template<size_t MERKLE_DEPTH, bool F>
    HashType ssz_restore_merkle_root(const HashType& leaf,
                                     const static_vector<HashType, MERKLE_DEPTH, F>& branch,
                                     uint64_t gindex,
                                     const uint64_t depth = MERKLE_DEPTH) {
        auto hash = leaf;

        for (size_t i = 0; i < depth; i++) {
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

            hash = parent_hash(left, right);
        }

        return hash;
    }

    template<size_t MERKLE_DEPTH, bool F>
    void ssz_verify_proof(const HashType& root,
                          const HashType& leaf,
                          const static_vector<HashType, MERKLE_DEPTH, F>& branch,
                          const uint64_t gindex,
                          const uint64_t depth = MERKLE_DEPTH) {
        auto expected_root = ssz_restore_merkle_root(leaf, branch, gindex, depth);
        assert_true(sha256_equals(root, expected_root));
    }

    HashType hash_tree_root(uint64_t val) {
#ifdef __ZKLLVM__
        // TODO: pack bytes into base_field_element here.
        HashType empty_hash_ = {0};
        return empty_hash_;
#else
        auto bytes = int_to_bytes<uint64_t, 32, true>(val);
        return bytes;
#endif
    }

    HashType hash_tree_root(const CheckpointVariable& checkpoint) {
        auto epoch_leaf = hash_tree_root(checkpoint.epoch);
        return parent_hash(epoch_leaf, checkpoint.root);
    }

    HashType hash_tree_root(const JustificationBitsVariable& checkpoint) {
#ifdef __ZKLLVM__
        // TODO: pack bytes into base_field_element here.
        HashType empty_hash_ = {0};
        return empty_hash_;
#else
        Bytes32 ret_val {};
        for (auto i = 0; i < 4; i++) {
            if (checkpoint.bits[i]) {
                ret_val[0] = set_nth_bit(ret_val[0], i);
            }
        }

        return ret_val;
#endif
    }

}    // namespace ssz_utils
