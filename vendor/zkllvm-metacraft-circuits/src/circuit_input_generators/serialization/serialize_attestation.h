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
nlohmann::json serialize<AttestationData>(const AttestationData& attestationData) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json(attestationData.slot));
    result["struct"].push_back(pack_int_json(attestationData.index));
    result["struct"].push_back(bytes32_to_hash_type(attestationData.beacon_block_root));
    result["struct"].push_back(serialize(attestationData.source));
    result["struct"].push_back(serialize(attestationData.target));
    return result;
}

AttestationData deserializeAttestationData(const nlohmann::json& j) {
    AttestationData ad;
    // ad.slot = bytes_to_int<decltype(ad.slot)>(bytes);
    // ad.index = bytes_to_int<decltype(ad.index)>(bytes);
    return ad;
}

template<>
nlohmann::json serialize<Validator>(const Validator& validator) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json((int)validator.trusted));
    result["struct"].push_back(pack_int_json(validator.validator_index));
    result["struct"].push_back(byte_array_to_json(validator.pubkey));
    result["struct"].push_back(byte_array_to_json(validator.withdrawal_credentials));
    result["struct"].push_back(pack_int_json(validator.effective_balance));
    result["struct"].push_back(pack_int_json(validator.slashed));
    result["struct"].push_back(pack_int_json(validator.activation_eligibility_epoch));
    result["struct"].push_back(pack_int_json(validator.activation_epoch));
    result["struct"].push_back(pack_int_json(validator.exit_epoch));
    result["struct"].push_back(pack_int_json(validator.withdrawable_epoch));
    result["struct"].push_back(serialize_vector(validator.validator_list_proof));
    return result;
}

template<>
nlohmann::json serialize<Fork>(const Fork& fork) {
    nlohmann::json result;
    result["struct"].push_back(byte_array_to_json(fork.previous_version));
    result["struct"].push_back(byte_array_to_json(fork.current_version));
    result["struct"].push_back(pack_int_json(fork.epoch));
    return result;
}

template<>
nlohmann::json serialize<Attestation>(const Attestation& attestation) {
    nlohmann::json result;
    result["struct"].push_back(serialize(attestation.data));
    result["struct"].push_back(byte_array_to_json(attestation.signature));
    result["struct"].push_back(serialize(attestation.fork));
    result["struct"].push_back(byte_array_to_json(attestation.genesis_validators_root));
    result["struct"].push_back(bytes32_to_hash_type(attestation.state_root));
    result["struct"].push_back(serialize_vector(attestation.state_root_proof));
    result["struct"].push_back(bytes32_to_hash_type(attestation.validators_root));
    result["struct"].push_back(serialize_vector(attestation.validators_root_proof));
    result["struct"].push_back(serialize_vector(attestation.validators));

    return result;
}