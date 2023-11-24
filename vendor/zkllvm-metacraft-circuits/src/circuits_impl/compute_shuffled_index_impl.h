#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "compute_shuffled_index_impl.h"
#include "../circuit_utils/circuit_byte_utils.h"
#include "../utils/picosha2.h"

using namespace circuit_byte_utils;

uint64_t compute_shuffled_index_impl(uint64_t index, uint64_t index_count, Bytes32 seed,
                                            int SHUFFLE_ROUND_COUNT = 90) {
    assert_true(index < index_count);

    Bytes32 source_buffer {};

    std::copy(seed.begin(), seed.end(), source_buffer.begin());

    // Swap or not (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
    // See the 'generalized domain' algorithm on page 3
    for (Byte current_round = 0; current_round < SHUFFLE_ROUND_COUNT; current_round++) {

        Bytes32 eth2digest_bytes = calc_hash(source_buffer, current_round);
        auto first8bytes = take_n_elements<Byte, eth2digest_bytes.size(), 8>(eth2digest_bytes);

        auto first8bytes_int = bytes_to_int<uint64_t>(first8bytes);
        auto pivot = first8bytes_int % index_count;
        uint64_t flip = (pivot + index_count - index) % index_count;
        auto position = std::max(index, flip);

        Bytes32 source_buffer_hash = calc_hash(source_buffer, current_round, int_to_bytes(uint32_t(position >> 8)));
        auto byte_value = source_buffer_hash[(position % 256) >> 3];
        auto bit = (byte_value >> (position % 8)) % 2;

        if (bit != 0) {
            index = flip;
        }
    }

    return index;
}
