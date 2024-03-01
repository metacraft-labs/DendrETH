#include "../circuits_impl/verify_attestation_data_imp.h"

[[circuit]] base_field_type verify_attestation_data(HashType block_root,
                                                    Attestation attestation,
                                                    base_field_type sigma
) {
    auto result = verify_attestation_data_imp(block_root, attestation, sigma);
    return result.token;
}
