#pragma once

#include <vector>
#include <algorithm>
#include <type_traits>

#include "json_serialization_utils.h"
#include "../../circuits_impl/verify_attestation_data_imp.h"

template<>
nlohmann::json serialize<CheckpointVariable>(const CheckpointVariable& checkpoint) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json(checkpoint.epoch));
    result["struct"].push_back(bytes32_to_hash_type(checkpoint.root));
    return result;
}

template<>
nlohmann::json serialize<AttestationData>(const AttestationData& ad) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json(ad.slot));
    result["struct"].push_back(pack_int_json(ad.index));
    result["struct"].push_back(bytes32_to_hash_type(ad.beacon_block_root));
    result["struct"].push_back(serialize<decltype(ad.source)>(ad.source));
    result["struct"].push_back(serialize<decltype(ad.target)>(ad.target));
    return result;
}

AttestationData deserializeAttestationData(const nlohmann::json& j) {
    AttestationData ad;
    // ad.slot = bytes_to_int<decltype(ad.slot)>(bytes);
    // ad.index = bytes_to_int<decltype(ad.index)>(bytes);
    return ad;
}

template<>
nlohmann::json serialize<Validator>(const Validator& v) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json((int)v.trusted));
    result["struct"].push_back(pack_int_json(v.validator_index));
    result["struct"].push_back(byte_array_to_json(v.pubkey));
    result["struct"].push_back(byte_array_to_json(v.withdrawal_credentials));
    result["struct"].push_back(pack_int_json(v.effective_balance));
    result["struct"].push_back(pack_int_json(v.slashed));
    result["struct"].push_back(pack_int_json(v.activation_eligibility_epoch));
    result["struct"].push_back(pack_int_json(v.activation_epoch));
    result["struct"].push_back(pack_int_json(v.exit_epoch));
    result["struct"].push_back(pack_int_json(v.withdrawable_epoch));
    result["struct"].push_back(serialize_vector(v.validator_list_proof));
    return result;
}
