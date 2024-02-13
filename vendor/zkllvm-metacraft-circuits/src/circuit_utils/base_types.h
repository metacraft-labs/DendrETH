#pragma once

#include <stdint.h>
#include <cstring>

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include "constants.h"

#ifdef __ZKLLVM__
#define assert_true(c) \
    { __builtin_assigner_exit_check(c); }
#else
#define assert_true(c) \
    { assert(c); }
#endif

#include "static_vector.h"

// This assertion is meant to be applied only when the code is compiled as executable.
// When compiling as circuit, it will have no effect for performance reasons.
#ifdef __ZKLLVM__
#define assert_in_executable(c)
#else
#define assert_in_executable(c) \
    { assert(c); }
#endif

using sha256_t = typename nil::crypto3::hashes::sha2<256>::block_type;

#ifdef __ZKLLVM__
using HashType = sha256_t;
#else
using HashType = Bytes32;
#endif

using Epoch = uint64_t;
using Slot = uint64_t;
using Root = HashType;
using Gwei = uint64_t;
template<size_t DEPTH>
using MerkleProof = static_vector<HashType, DEPTH, true>;
using BeaconStateLeafProof = MerkleProof<5>;

#define countof(array) (sizeof(array) / sizeof(array[0]))

bool sha256_equals(sha256_t hash1, sha256_t hash2) {
    bool result = true;
    for (auto i = 0; i < countof(hash1); ++i) {
        result = result && (hash1[i] == hash2[i]);
    }

    return result;
}

bool sha256_equals(Bytes32 hash1, Bytes32 hash2) {
    return hash1 == hash2;
}

struct CheckpointVariable {
    Epoch epoch;
    Root root;
    bool operator==(const CheckpointVariable &c) const {
        return (epoch == c.epoch && sha256_equals(root, c.root));
    }
} __attribute__((packed));

struct JustificationBitsVariable {

    static_vector<bool, 4, true> bits;

    constexpr JustificationBitsVariable(std::initializer_list<bool> init) { 
        size_t i = 0;
        for(const auto& v : init) {
            assert_true(i < bits.size());
            bits[i++] = v;
        }
    }

    constexpr JustificationBitsVariable() { 
        for(size_t i = 0; i < bits.size(); ++i) {
            bits[i] = false;
        }
    }

    void shift_left(size_t n) {
        assert_in_executable(n > 0);
        assert_in_executable(n <= bits.size());
        memmove(&bits[0], &bits[n], sizeof(bool) * (bits.size() - n));
        memset(&bits[bits.size() - n], 0, sizeof(bool) * n);
    }
    void shift_right(size_t n) {
        assert_in_executable(n > 0);
        assert_in_executable(n <= bits.size());
        memmove(&bits[n], &bits[0], sizeof(bool) * (bits.size() - n));
        memset(&bits[0], 0, sizeof(bool) * n);
    }
    bool test_range(const size_t lower_bound, const size_t upper_bound_non_inclusive) const {
        assert_in_executable(lower_bound < upper_bound_non_inclusive);
        assert_in_executable(lower_bound >= 0);
        assert_in_executable(upper_bound_non_inclusive <= bits.size());
        bool result = true;
        for (size_t i = lower_bound; i < upper_bound_non_inclusive; i++) {
            result = result && bits[i];
        }
        return result;
    }
    bool operator==(const JustificationBitsVariable &j) const {
        return (bits == j.bits);
    }
} __attribute__((packed));

Epoch get_current_epoch(Slot slot) {
    return slot / SLOTS_PER_EPOCH;
}
