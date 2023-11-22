#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "base_types.h"

using namespace nil::crypto3;

namespace circuit_byte_utils {

#define countof(array) (sizeof(array) / sizeof(array[0]))

    using sha256_t = typename hashes::sha2<256>::block_type;

    bool is_same(sha256_t block0, sha256_t block1) {

        bool result = true;
        for (auto i = 0; i < countof(block0) && result; ++i) {
            result = result && (block0[i] == block1[i]);
        }

        return result;
    }

    template<typename T>
    Byte get_nth_byte(const T& val, unsigned int n) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        assert_true(n < sizeof(T));

        return val >> (n * 8);
    }

    bool get_nth_bit(uint64_t value, short i) {
        return 1 & (value >> i);
    }

    Byte set_nth_bit(Byte& value, short i) {
        assert_true(i < 8);
        value = value | (Byte(1) << i);
        return value;
    }

    template<typename T>
    void sha256_to_bytes_array(sha256_t sha, T& out) {
        assert_true(out.size() >= sizeof(sha));
        for (int int_count = 0; int_count < sizeof(sha) / sizeof(sha[0]); int_count++) {

            for (int byte_count = 0; byte_count < sizeof(sha[0]); byte_count++) {
                out[int_count * sizeof(sha[0]) + byte_count] = get_nth_byte(sha[int_count], byte_count);
            }
        }
    }

    template<typename T, std::size_t inCount, std::size_t N>
    std::array<T, N> take_n_elements(const std::array<T, inCount>& val) {
        static_assert(N <= inCount);
        std::array<T, N> ret {};
        for (auto i = 0u; i < N; ++i) {
            ret[i] = val[i];
        }
        return ret;
    }

    template<typename T>
    std::array<Byte, sizeof(T)> int_to_bytes(const T& paramInt, bool little_endian = true) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        std::array<Byte, sizeof(T)> bytes {};
        if (little_endian) {
            for (int i = 0; i < sizeof(T); ++i) {
                bytes[i] = (paramInt >> (i * 8));
            }
        } else {
            for (int i = sizeof(T) - 1; i >= 0; i--) {
                bytes[i] = (paramInt >> (i * 8));
            }
        }
        return bytes;
    }

    template<typename T>
    T bytes_to_int(const std::array<Byte, sizeof(T)>& paramVec, bool little_endian = true) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        T result = 0;
        if (little_endian) {
            for (int i = sizeof(T) - 1; i >= 0; i--) {
                result = (result << 8) + paramVec[i];
            }
        } else {
            for (unsigned i = 0; i < sizeof(T); ++i) {
                result = (result << 8) + paramVec[i];
            }
        }
        return result;
    }

}    // namespace circuit_byte_utils
