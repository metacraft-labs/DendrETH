#include "../circuits_imp/compute_shuffled_index_imp.h"

[[circuit]] uint64_t compute_shuffled_index(uint64_t index, uint64_t index_count, std::array<Byte, 32> seed,
                                            int SHUFFLE_ROUND_COUNT = 90) {
    return compute_shuffled_index_imp(index, index_count, seed, SHUFFLE_ROUND_COUNT);
}
