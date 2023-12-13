#pragma once

#include <stdint.h>
#include <cstring>
#include "constants.h"

#ifdef __ZKLLVM__
#define assert_true(c) \
    { __builtin_assigner_exit_check(c); }
#else
#define assert_true(c) \
    { assert(c); }
#endif

// This assertion is meant to be applied only when the code is compiled as executable.
// When compiling as circuit, it will have no effect for performance reasons.
#ifdef __ZKLLVM__
#define assert_in_executable(c)
#else
#define assert_in_executable(c) \
    { assert(c); }
#endif

using Byte = unsigned char;
using Bytes32 = std::array<Byte, 32>;
using Bytes48 = std::array<Byte, 48>;
using Bytes64 = std::array<Byte, 64>;

using Epoch = uint64_t;
using Slot = uint64_t;
using Root = Bytes32;
using Gwei = uint64_t;
template<size_t DEPTH>
using MerkleProof = std::array<Bytes32, DEPTH>;
using BeaconStateLeafProof = MerkleProof<5>;

struct CheckpointVariable {
    Epoch epoch;
    Root root;
    bool operator==(const CheckpointVariable &c) const {
        return (epoch == c.epoch && root == c.root);
    }
};

struct JustificationBitsVariable {

    std::array<bool, 4> bits;

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
};

Epoch get_current_epoch(Slot slot) {
    return slot / SLOTS_PER_EPOCH;
}
