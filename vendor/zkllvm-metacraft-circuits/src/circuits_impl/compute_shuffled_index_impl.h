#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "../circuit_utils/circuit_byte_utils.h"
#include "../utils/picosha2.h"

using namespace circuit_byte_utils;

uint64_t compute_shuffled_index_impl(uint64_t index, uint64_t index_count, Bytes32 seed, int shuffle_round_count) {
    assert_true(index < index_count);

    // Swap or not (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
    // See the 'generalized domain' algorithm on page 3
    for (Byte current_round = 0; current_round < shuffle_round_count; current_round++) {

        auto pivot = bytes_to_int<uint64_t>(take<8>(sha256(seed, current_round))) % index_count;
        uint64_t flip = (pivot + index_count - index) % index_count;
        auto position = std::max(index, flip);

        Bytes32 seed_hash = sha256(seed, current_round, int_to_bytes(uint32_t(position / 256)));
        auto byte = seed_hash[(position % 256) / 8];
        auto bit = (byte >> (position % 8)) % 2;

        if (bit == 1) {
            index = flip;
        }
    }

    return index;
}
