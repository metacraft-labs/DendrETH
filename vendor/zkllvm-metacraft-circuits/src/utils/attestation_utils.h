#pragma once

#include "json/json.hpp"
using namespace nlohmann;

#include "byte_utils.h"
using namespace byte_utils;

#include "circuits_impl/verify_attestation_data_imp.h"

namespace attestation_utils {

AttestationData parse_attestation_data(const json& json_attestation_data) {
    AttestationData attestation_data;
    attestation_data.slot = json_attestation_data["slot"];
    attestation_data.index = json_attestation_data["index"];
    attestation_data.beacon_block_root = hexToBytes<32>(json_attestation_data["beacon_block_root"]);
    attestation_data.source.epoch = json_attestation_data["source"]["epoch"];
    attestation_data.source.root = hexToBytes<32>(json_attestation_data["source"]["root"]);
    attestation_data.target.epoch = json_attestation_data["target"]["epoch"];
    attestation_data.target.root = hexToBytes<32>(json_attestation_data["target"]["root"]);
    return attestation_data;
}

Fork parse_fork(const json& json_fork) {
    Fork fork;
    fork.current_version = circuit_byte_utils::expand<32>(hexToBytes<4>(json_fork["current_version"]));
    fork.previous_version = circuit_byte_utils::expand<32>(hexToBytes<4>(json_fork["previous_version"]));
    fork.epoch = json_fork["epoch"];
    return fork;
}

Validator parse_validator(const json& json_validator) {
    Validator validator;
    validator.trusted = json_validator["trusted"];
    validator.validator_index = json_validator["validator_index"];
    validator.pubkey = hexToBytes<48>(json_validator["pubkey"]);
    std::string withdrawal_credentials;
    if (json_validator.contains("withdrawal_credentials")) {
        withdrawal_credentials = json_validator["withdrawal_credentials"];
    } else {
        withdrawal_credentials.assign(64, '0');
    }
    validator.withdrawal_credentials = hexToBytes<32>(withdrawal_credentials);
    if (json_validator.contains("effective_balance")) {
        validator.effective_balance = json_validator["effective_balance"];
    } else {
        validator.effective_balance = 0;
    }
    if (json_validator.contains("slashed")) {
        validator.slashed = json_validator["slashed"];
    } else {
        validator.slashed = false;
    }
    if (json_validator.contains("activation_eligibility_epoch")) {
        validator.activation_eligibility_epoch = json_validator["activation_eligibility_epoch"];
    } else {
        validator.activation_eligibility_epoch = 0;
    }
    if (json_validator.contains("activation_epoch")) {
        validator.activation_epoch = json_validator["activation_epoch"];
    } else {
        validator.activation_epoch = 0;
    }
    if (json_validator.contains("exit_epoch")) {
        validator.exit_epoch = json_validator["exit_epoch"];
    } else {
        validator.exit_epoch = 0;
    }
    if (json_validator.contains("withdrawable_epoch")) {
        validator.withdrawable_epoch = json_validator["withdrawable_epoch"];
    } else {
        validator.withdrawable_epoch = 0;
    }
    if (json_validator.contains("validator_list_proof")) {
        for (size_t i = 0; i < json_validator["validator_list_proof"].size(); i++) {
            std::string element = json_validator["validator_list_proof"][i];
            if (element.size() == 0) {
                element.assign(64, '0');
            }
            validator.validator_list_proof.push_back(hexToBytes<32>(element));
        }
    }
    return validator;
}

Attestation parse_attestation(const json& json_attestation) {
    Attestation attestation;
    attestation.data = parse_attestation_data(json_attestation["data"]);
    attestation.signature = hexToBytes<96>(json_attestation["signature"]);
    attestation.fork = parse_fork(json_attestation["fork"]);
    attestation.genesis_validators_root = hexToBytes<32>(json_attestation["genesis_validators_root"]);
    attestation.state_root = hexToBytes<32>(json_attestation["state_root"]);
    for (size_t i = 0; i < json_attestation["state_root_proof"].size(); i++) {
        attestation.state_root_proof[i] = hexToBytes<32>(json_attestation["state_root_proof"][i]);
    }
    attestation.validators_root = hexToBytes<32>(json_attestation["validators_root"]);
    for (size_t i = 0; i < json_attestation["validators_root_proof"].size(); i++) {
        attestation.validators_root_proof[i] = hexToBytes<32>(json_attestation["validators_root_proof"][i]);
    }
    for (size_t i = 0; i < json_attestation["validators"].size(); i++) {
        attestation.validators.push_back(parse_validator(json_attestation["validators"][i]));
    }
    return attestation;
}

}