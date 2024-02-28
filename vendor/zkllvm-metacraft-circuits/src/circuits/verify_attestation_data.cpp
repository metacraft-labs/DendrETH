#include "../circuits_impl/verify_attestation_data_imp.h"

[[circuit]] int64_t verify_attestation_data(AttestationData ad
                                            //   std::array<PubKey, 1'000'000> trustedKeys
                                            //   size_t pubkeysCount,
                                            //   int64_t sigma
                                            ) {

    // assert_true(trustedKeys.size() >= pubkeysCount);
    // base_field_type reconstructed_token;
    // uint64_t result = 0;

    // PubKey& pk = trustedKeys[0];

    // for(size_t i = 0; i < pk.size(); i++) {
    //     result += pk[i];
    // }

    auto result = ad.slot + ad.index + ad.source.epoch + ad.target.epoch;

    return result;
    // return process_votes(trustedKeys.data(), pubkeysCount, sigma, reconstructed_token);
}

// [[circuit]] int64_t verify_attestation_data(CheckpointVariable cp
//                                             //   std::array<PubKey, 1'000'000> trustedKeys
//                                             //   size_t pubkeysCount,
//                                             //   int64_t sigma
//                                             ) {

//     // assert_true(trustedKeys.size() >= pubkeysCount);
//     // base_field_type reconstructed_token;
//     // uint64_t result = 0;

//     // PubKey& pk = trustedKeys[0];

//     // for(size_t i = 0; i < pk.size(); i++) {
//     //     result += pk[i];
//     // }

//     auto result = cp.epoch + 111;

//     return result;
//     // return process_votes(trustedKeys.data(), pubkeysCount, sigma, reconstructed_token);
// }