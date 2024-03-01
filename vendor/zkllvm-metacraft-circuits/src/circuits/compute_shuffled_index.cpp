#include "../circuits_impl/compute_shuffled_index_impl.h"

[[circuit]] uint64_t compute_shuffled_index(uint64_t index, uint64_t index_count, Bytes32 seed,
                                            int shuffle_round_count) {
    return compute_shuffled_index_impl(index, index_count, seed, shuffle_round_count);
}
