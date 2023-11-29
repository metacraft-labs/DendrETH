#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "../circuit_utils/circuit_byte_utils.h"
#include "../utils/picosha2.h"

using namespace circuit_byte_utils;

struct AttestationData {
    int slot;
    int index;
    Root beacon_block_root;
    CheckpointVariable source;
    CheckpointVariable target;
};

struct Fork {
    Bytes32 previous_version;
    Bytes32 current_version;
    int epoch;
};

struct Validator {
    // True if it can be proven that this validator is a member of the
    // source state and can, therefore, be trusted.
    bool trusted;
    // These fields are always present.
    int validator_index;
    Bytes32 pubkey;
    // These fields are present iff `trusted` is True.
    Bytes32 withdrawal_credentials;
    int effective_balance;
    bool slashed;
    int activation_eligibility_epoch;
    int activation_epoch;
    int exit_epoch;
    int withdrawable_epoch;
    Proof validator_list_proof;
}

struct Attestation {
    // Standard attestation data.
    AttestationData data;
    Bytes32 signature;

    // Needed to compute the `signing_root` and verify the `signature`.
    Fork fork;
    Bytes32 genesis_validators_root;

    // We should be able to prove that the majority of validators
    // participating in this attestation are part of the validator set
    // associated with the state of the last trusted block.
    Bytes32 state_root;
    MerkleProof<48> state_root_proof;

    Bytes32 validators_root;
    MerkleProof<48> validators_root_proof;

    // std::array<> validators: List[Validator]
};