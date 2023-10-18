using namespace nil::crypto3;

namespace byte_utils {

    using sha256_t = typename hashes::sha2<256>::block_type;

    #ifdef __ZKLLVM__
    #define assert_true(c) {                 \
        __builtin_assigner_exit_check(c);    \
    }
    #else
    #define assert_true(c) {                 \
        assert(c);                           \
    }
    #endif

    bool is_same(sha256_t block0,
        sha256_t block1){

        bool result = true;
        for(auto i = 0; i < sizeof(block0)/sizeof(block0[0]) && result; i++) {
            printf("Element fount %d\n", i);
            result = result && (block0[0] == block1[0]);
        }

        return result;
    }

    template <typename T>
    char get_nth_byte(const T& val, unsigned int n) {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        assert_true(n < sizeof(T));

        return val >> (n * 8);
    }

    template <typename T>
    void sha256_to_bytes_array(sha256_t sha, T& out) {
        assert_true(out.size() >= sizeof(sha));
        for(int int_count = 0; int_count < sizeof(sha)/sizeof(sha[0]); int_count++) {

            for(int byte_count = 0; byte_count < sizeof(sha[0]); byte_count++) {
                out[int_count * sizeof(sha[0]) + byte_count] = get_nth_byte<decltype(sha[int_count])>(sha[int_count], byte_count);
            }

        }
    }

    template <typename T, std::size_t inCount, std::size_t N>
    std::array<T, N> take_n_elements(const std::array<T, inCount>& val) {
        static_assert(N <= inCount);
        std::array<T, N> ret{};
        for(auto i = 0u; i < N; i++) {
            ret[i] = val[i];
        }
        return ret;
    }

    template <typename T>
    std::array<unsigned char, sizeof(T)> int_to_bytes(const T& paramInt)
    {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        std::array<unsigned char, sizeof(T)> arrayOfByte{};
        for (int i = 0; i < sizeof(T); i++) {
            arrayOfByte[sizeof(T) - 1 - i] = get_nth_byte(paramInt, i);
        }
        return arrayOfByte;
    }

    template <typename T>
    T bytes_to_int(const std::array<unsigned char, sizeof(T)>& paramVec)
    {
        static_assert(std::is_integral<typename std::remove_reference<T>::type>::value, "T must be integral");
        T val = 0;
        for (int i = sizeof(T) - 1; i >= 0; i--) {
            int temp = paramVec[i];
            val |= (temp << ((sizeof(T) - 1 - i) * 8));
        }
        return val;
    }

}
