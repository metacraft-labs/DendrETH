#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "../circuit_utils/circuit_byte_utils.h"
#include "../circuit_utils/ssz_utils.h"
#include "../utils/picosha2.h"
#include "../circuit_utils/static_vector.h"

using namespace circuit_byte_utils;
using namespace ssz_utils;

using Proof = static_vector<Bytes32>;

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
    // These fields are meaningful iff `trusted` is True.
    Bytes32 withdrawal_credentials;
    int effective_balance;
    bool slashed;
    int activation_eligibility_epoch;
    int activation_epoch;
    int exit_epoch;
    int withdrawable_epoch;
    Proof validator_list_proof;
};

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
    MerkleProof<3> state_root_proof;

    Bytes32 validators_root;
    MerkleProof<5> validators_root_proof;

    static_vector<Validator> validators;
};

struct Transition {
    CheckpointVariable source;
    CheckpointVariable target;
};

struct TransitionKeys {
    Transition transition;
    static_vector<Bytes32> keys;
};

struct Merged {
    static_vector<Attestation> attestations;
    static_vector<TransitionKeys> trusted_pubkeys;
};

struct VoteToken {
    Transition transition;
    int count;
};

using TransitionKey = Bytes32;

static_vector<Bytes32> compute_zero_hashes(int length = 64) {
    static_vector<Bytes32> xs;
    xs.push_back(get_empty_byte_array<32>());
    for(int i = 1; i < length; i++) {
        xs.push_back(calc_hash(xs[i-1], xs[i-1]));
    }
    return xs;
}

static const auto zero_hashes = compute_zero_hashes();

static_vector<Bytes32> fill_zero_hashes(
        const static_vector<Bytes32>& xs,
        size_t length = 0
)
{
    static_vector<Bytes32> ws = xs;
    int additions_count = length - xs.size();
    for(int i = additions_count; i > 0; i--) {
        ws.push_back(zero_hashes[xs.size() + additions_count - i]);
    }
    return ws;
}

VoteToken verify_attestation_data(Bytes32 block_root, Attestation attestation, int sigma) {
    assert_true(sigma != 0);

    ssz_verify_proof(
        block_root,
        attestation.state_root,
        fill_zero_hashes(attestation.state_root_proof).content(),
        11,
        3
    );

    ssz_verify_proof(
        attestation.state_root,
        attestation.validators_root,
        fill_zero_hashes(attestation.validators_root_proof).content(),
        43,
        5
    );

    return {};
}
