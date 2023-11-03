#pragma once

#include <stdint.h>

using Byte = unsigned char;
using Bytes32 = std::array<Byte, 32>;
using Bytes64 = std::array<Byte, 64>;

template<typename>
struct array_size;
template<typename T, size_t N>
struct array_size<std::array<T,N> > {
    static size_t const size = N;
};

using Epoch = uint64_t;
using Slot = uint64_t;
using Root = Bytes32;
using Gwei = uint64_t;
template <size_t DEPTH>
using MerkleProof = std::array<Bytes32, DEPTH>;
using BeaconStateLeafProof = MerkleProof<5>;

struct CheckpointVariable {
    Epoch epoch;
    Root root;
};

constexpr unsigned int MAX_MERKLE_DEPTH = 512;

//!!!TODO: Use assertion in circuits when introduced in tooling
#ifdef __ZKLLVM__
#define assert_true(c)
#else
#define assert_true(c) \
    { assert(c); }
#endif