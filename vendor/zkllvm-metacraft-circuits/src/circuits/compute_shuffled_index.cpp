#include "../circuits_impl/compute_shuffled_index_impl.h"
#include <array>


[[circuit]] uint64_t compute_shuffled_index(uint64_t index, uint64_t index_count, std::array<Byte, 32> seed_,
                                            int SHUFFLE_ROUND_COUNT) {
    Bytes32 seed(seed_);
    return compute_shuffled_index_impl(index, index_count, seed, SHUFFLE_ROUND_COUNT);
}
