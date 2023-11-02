#include <stdint.h>

#pragma once

using Byte = unsigned char;
using Bytes32 = std::array<Byte, 32>;

using Epoch = uint64_t;
using Slot = uint64_t;
using Root = Bytes32;
using Gwei = uint64_t;
template <size_t DEPTH>
using MerkleProof = std::array<Bytes32, DEPTH>;
using BeaconStateLeafProof = MerkleProof<5>;

constexpr unsigned int MAX_MERKLE_DEPTH = 512;

//!!!TODO: Use assertion in circuits when introduced in tooling
#ifdef __ZKLLVM__
#define assert_true(c)
#else
#define assert_true(c) \
    { assert(c); }
#endif