#include "base_types.h"

using namespace nil::crypto3;

namespace circuit_byte_utils {

    using sha256_t = typename hashes::sha2<256>::block_type;

#ifdef __ZKLLVM__
#define assert_true(c) \
    { __builtin_assigner_exit_check(c); }
#else
#define assert_true(c) \
    { assert(c); }
#endif

    bool is_same(sha256_t block0, sha256_t block1) {

        bool result = true;
        for (auto i = 0; i < sizeof(block0) / sizeof(block0[0]) && result; i++) {
            printf("Element found %d\n", i);
            result = result && (block0[0] == block1[0]);
        }

        return result;
    }

    template<typename T>
    char get_nth_byte(const T& val, unsigned int n) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        assert_true(n < sizeof(T));

        return val >> (n * 8);
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
        for (auto i = 0u; i < N; i++) {
            ret[i] = val[i];
        }
        return ret;
    }

    template<typename T>
    std::array<Byte, sizeof(T)> int_to_bytes(const T& paramInt, bool little_endian = true) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        std::array<Byte, sizeof(T)> arrayOfByte {};
        if (little_endian) {
            for (int i = 0; i < sizeof(T); i++) {
                arrayOfByte[i] = (paramInt >> (i * 8));
            }
        } else {
            for (int i = sizeof(T) - 1; i >= 0; i--) {
                arrayOfByte[i] = (paramInt >> (i * 8));
            }
        }
        return arrayOfByte;
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
            for (unsigned i = 0; i < sizeof(T); i++) {
                result = (result << 8) + paramVec[i];
            }
        }
        return result;
    }

}    // namespace circuit_byte_utils
