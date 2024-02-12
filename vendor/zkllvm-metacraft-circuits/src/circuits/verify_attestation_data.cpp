#include "../circuits_impl/verify_attestation_data_imp.h"

[[circuit]] int64_t verify_attestation_data(std::array<PubKey, 1'000'000> trustedKeys
                                            //   size_t pubkeysCount,
                                            //   int64_t sigma
                                            ) {

    // assert_true(trustedKeys.size() >= pubkeysCount);
    base_field_type reconstructed_token;
    uint64_t result = 0;

    PubKey& pk = trustedKeys[0];

    for(size_t i = 0; i < pk.size(); i++) {
        result += pk[i];
    }

    return result;
    // return process_votes(trustedKeys.data(), pubkeysCount, sigma, reconstructed_token);
}
