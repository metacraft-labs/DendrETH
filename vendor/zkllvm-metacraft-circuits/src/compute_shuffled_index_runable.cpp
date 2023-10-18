#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "byte_utils.h"

using namespace byte_utils;

constexpr unsigned char SHUFFLE_ROUND_COUNT = 90;

uint64_t compute_shuffled_index(
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

int main(int argc, char* argv[]) {

    for(int i = 0; i < 10000; i++) {
        int val = rand();
        printf("starting val = %d\n", val);
        int saved = val;
        auto myArr = int_to_bytes(val);
        val = bytes_to_int<int>(myArr);
        printf("after the convertions val = %d\n\n", val);    

        assert_true(val == saved);
    }

    for(int i = 0; i < 10000; i++) {
        uint64_t val = rand();
        printf("starting val64 = %ld\n", val);
        int saved = val;
        auto myArr = int_to_bytes(val);
        val = bytes_to_int<uint64_t>(myArr);
        printf("after the convertions val64 = %ld\n\n", val);

        assert_true(val == saved);
    }

    for(int i = 0; i < 10000; i++) {
        unsigned char val = rand();
        printf("starting unsigned char = %d\n", val);
        int saved = val;
        auto myArr = int_to_bytes(val);
        val = bytes_to_int<unsigned char>(myArr);
        printf("after the convertions unsigned char = %d\n\n", val);    

        assert_true(val == saved);
    }


// ##################################################################################################################
    sha256_t seed = {1, 2, 3, 4, 5, 6, 7, 8, 255, 256, 11, 65536, 16777215, 16777216, 167772160, 1677721600};
    std::array<unsigned char, 16*4> source_buffer;

    sha256_to_bytes_array(seed, source_buffer);

    printf("\nDEBUG source_buffer:");
    for(int i = 0; i < 16*4; i++) {
        printf("%d ", (int)source_buffer[i]);
    }
    printf("\n");
// ##################################################################################################################

    std::array<bool, 5> a = {0, 1, 1, 0, 1};
    auto b = take_n_elements<bool, a.size(), 4>(a);
    hashes::sha2<256>::digest_type d = hash<hashes::sha2<256>>(b.begin(), b.end());

    for(auto it = b.begin(); it != b.end(); it++) {
        printf("DEBUG B: %d\n", (int)*it);
    }

// ##################################################################################################################

    // compute_shuffled_index(3, 5, b1);

    return 0;
}
