#pragma once

#include <stdint.h>
#include <cstring>
#include "constants.h"

//!!!TODO: Use assertion in circuits when introduced in tooling
#ifdef __ZKLLVM__
#define assert_true(c)
#else
#define assert_true(c) \
    { assert(c); }
#endif

using Byte = unsigned char;
using Bytes32 = std::array<Byte, 32>;
using Bytes64 = std::array<Byte, 64>;

template<typename>
struct array_size;
template<typename T, size_t N>
struct array_size<std::array<T, N>> {
    static size_t const size = N;
};

using Epoch = uint64_t;
using Slot = uint64_t;
using Root = Bytes32;
using Gwei = uint64_t;
template<size_t DEPTH>
using MerkleProof = std::array<Bytes32, DEPTH>;
using BeaconStateLeafProof = MerkleProof<5>;

using AttestedValidators = std::array<bool, 16'000'000>;

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
        assert_true(n > 0);
        assert_true(n <= bits.size());
        memmove(&bits[0], &bits[n], sizeof(bool) * (bits.size() - n));
        memset(&bits[bits.size() - n], 0, sizeof(bool) * n);
    }
    void shift_right(size_t n) {
        assert_true(n > 0);
        assert_true(n <= bits.size());
        memmove(&bits[n], &bits[0], sizeof(bool) * (bits.size() - n));
        memset(&bits[0], 0, sizeof(bool) * n);
    }
    bool test_range(const size_t lower_bound, const size_t upper_bound_non_inclusive) const {
        assert_true(lower_bound < upper_bound_non_inclusive);
        assert_true(lower_bound >= 0);
        assert_true(upper_bound_non_inclusive <= bits.size());
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

constexpr unsigned int MAX_MERKLE_DEPTH = 512;

Epoch get_current_epoch(Slot slot) {
    auto slots_per_epoch = SLOTS_PER_EPOCH;
    return slot / slots_per_epoch;
}