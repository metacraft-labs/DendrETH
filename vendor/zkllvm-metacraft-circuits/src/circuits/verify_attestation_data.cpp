#include "../circuits_impl/verify_attestation_data_imp.h"

[[circuit]] int64_t verify_attestation_data(std::array<PubKey, 1'000'000> trustedKeys,
                                              size_t pubkeysCount,
                                              int64_t sigma) {
    assert_true(trustedKeys.size() >= pubkeysCount);
    base_field_type reconstructed_token;

    // return process_votes(trustedKeys.data(), pubkeysCount, sigma, reconstructed_token);
    return 0;
}