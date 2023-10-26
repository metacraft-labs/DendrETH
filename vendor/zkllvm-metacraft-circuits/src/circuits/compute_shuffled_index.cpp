#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "../circuit_utils/circuit_byte_utils.h"

using namespace circuit_byte_utils;

constexpr unsigned char SHUFFLE_ROUND_COUNT = 90;

[[circuit]] uint64_t compute_shuffled_index(
        uint64_t index,
        uint64_t index_count,
        sha256_t seed) {
    assert_true(index < index_count);

    std::array<unsigned char, 32+1+4> source_buffer;
    uint64_t cur_idx_permuted = index;

    sha256_to_bytes_array(seed, source_buffer);

    // Swap or not (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
    // See the 'generalized domain' algorithm on page 3
    for(unsigned char current_round = 0; current_round < SHUFFLE_ROUND_COUNT; current_round++) {
        source_buffer[32] = current_round;

        auto eth2digest = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.begin() + 33);
        std::array<unsigned char, 32> eth2digest_bytes;
        sha256_to_bytes_array(eth2digest, eth2digest_bytes);
        auto first8bytes = take_n_elements<unsigned char, eth2digest_bytes.size(), 8>(eth2digest_bytes);
        auto first8bytes_int = bytes_to_int<uint64_t>(first8bytes);
        auto pivot = first8bytes_int % index_count;
        auto flip = ((index_count + pivot) - cur_idx_permuted) % index_count;
        auto position = std::max(cur_idx_permuted, flip);

        auto source_buffer_additional_bytes = int_to_bytes(uint32_t(position >> 8));
        for (auto i = 0; i <= 4; i++) {
            source_buffer[33 + i] = source_buffer_additional_bytes[i];
        }

        auto source = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.end());
        std::array<unsigned char, 32> source_to_bytes;
        sha256_to_bytes_array(source, source_to_bytes);
        auto byte_value = source_to_bytes[(position % 256) >> 3];
        auto bit = (byte_value >> (position % 8)) % 2;

        if(bit != 0) {
            cur_idx_permuted = flip;
        }
    }

    return cur_idx_permuted;

}
