#include <vector>
#include <algorithm>

#include "../../circuits_impl/verify_attestation_data_imp.h"

std::vector<unsigned char> serializeAttestationData(const AttestationData& ad) {
    std::vector<unsigned char> result;
    auto slot_bytes = int_to_bytes(ad.slot);
    std::copy(slot_bytes.begin(), slot_bytes.end(), std::back_inserter(result));
    auto index_bytes = int_to_bytes(ad.index);
    std::copy(index_bytes.begin(), index_bytes.end(), std::back_inserter(result));
    return result;
}

AttestationData deserializeAttestationData(const std::vector<unsigned char>& bytes) {
    AttestationData ad;
    // ad.slot = bytes_to_int<decltype(ad.slot)>(bytes);
    // ad.index = bytes_to_int<decltype(ad.index)>(bytes);
    return ad;
}