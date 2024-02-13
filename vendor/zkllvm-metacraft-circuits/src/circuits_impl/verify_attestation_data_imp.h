#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/algebra/curves/pallas.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <array>

#include "../circuit_utils/circuit_byte_utils.h"
#include "../circuit_utils/ssz_utils.h"
#include "../utils/picosha2.h"
#include "../circuit_utils/static_vector.h"

using namespace circuit_byte_utils;
using namespace ssz_utils;
using namespace nil::crypto3::algebra::curves;

using Proof = static_vector<HashType, 41>;
using PubKey = Bytes48;

struct AttestationData {
    uint64_t slot;
    uint64_t index;
    Root beacon_block_root;
    CheckpointVariable source;
    CheckpointVariable target;
} __attribute__((packed));

struct Fork {
    Bytes32 previous_version;
    Bytes32 current_version;
    uint64_t epoch;
} __attribute__((packed));

struct Validator {
    // True if it can be proven that this validator is a member of the
    // source state and can, therefore, be trusted.
    bool trusted;
    // These fields are always present.
    uint64_t validator_index;
    PubKey pubkey;
    // These fields are meaningful iff `trusted` is True.
    Bytes32 withdrawal_credentials;
    uint64_t effective_balance;
    bool slashed;
    uint64_t activation_eligibility_epoch;
    uint64_t activation_epoch;
    uint64_t exit_epoch;
    uint64_t withdrawable_epoch;
    Proof validator_list_proof;
} __attribute__((packed));

struct Attestation {
    // Standard attestation data.
    AttestationData data;
    Bytes96 signature;

    // Needed to compute the `signing_root` and verify the `signature`.
    Fork fork;
    Bytes32 genesis_validators_root;

    // We should be able to prove that the majority of validators
    // participating in this attestation are part of the validator set
    // associated with the state of the last trusted block.
    HashType state_root;
    MerkleProof<3> state_root_proof;

    HashType validators_root;
    MerkleProof<5> validators_root_proof;

    static_vector<Validator, 415> validators;
} __attribute__((packed));

struct Transition {
    CheckpointVariable source;
    CheckpointVariable target;
    bool operator==(const Transition& c) const {
        return (source == c.source && target == c.target);
    }
} __attribute__((packed));

struct TransitionKeys {
    Transition transition;
    static_vector<Bytes32> keys;
} __attribute__((packed));

struct Merged {
    static_vector<Attestation> attestations;
    static_vector<TransitionKeys> trusted_pubkeys;
} __attribute__((packed));

#ifdef __ZKLLVM__
using base_field_type = typename pallas::base_field_type::value_type;
#else
using base_field_type = uint64_t;
#endif

struct VoteToken {
    Transition transition;
    base_field_type token;
};

using TransitionKey = Bytes32;

static_vector<HashType> compute_zero_hashes(int length = 64) {
    static_vector<HashType> xs;
#ifdef __ZKLLVM__
    sha256_t empty_hash = {0};
    xs.push_back(empty_hash);
#else
    xs.push_back(get_empty_byte_array<32>());
#endif

    for (int i = 1; i < length; i++) {
        xs.push_back(parent_hash(xs[i - 1], xs[i - 1]));
    }
    return xs;
}


Proof fill_zero_hashes(const Proof& xs, size_t length = 0) {
    const auto zero_hashes = compute_zero_hashes();
#ifdef __ZKLLVM__
    sha256_t empty_hash = {0};
#else
    auto empty_hash = get_empty_byte_array<32>();
#endif
    Proof ws = xs;
    int additions_count = length - xs.size();

    for (int i = 0; i < ws.size(); i++) {
        if (sha256_equals(ws[i], empty_hash)) {
            ws[i] = zero_hashes[i];
        }
    }
    for (int i = additions_count; i > 0; i--) {
        ws.push_back(zero_hashes[xs.size() + additions_count - i]);
    }
    return ws;
}

HashType hash_validator(Bytes64 pubkey,
                        Bytes32 withdrawal_credentials_,
                        uint64_t effective_balance_,
                        uint64_t activation_eligibility_epoch_,
                        uint64_t activation_epoch_,
                        uint64_t exit_epoch_,
                        uint64_t withdrawable_epoch_) {
    // Convert parameters.
    auto effective_balance = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(effective_balance_)));
    auto slashed = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(0)));
    auto activation_eligibility_epoch = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(activation_eligibility_epoch_)));
    auto activation_epoch = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(activation_epoch_)));
    auto exit_epoch = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(exit_epoch_)));
    auto withdrawable_epoch = bytes_to_hash_type((int_to_bytes<uint64_t, 32, true>(withdrawable_epoch_)));
    auto withdrawal_credentials = bytes_to_hash_type((withdrawal_credentials_));

    #ifdef __ZKLLVM__
    auto hash = [](const HashType& lhs, const HashType& rhs) {
        return parent_hash(lhs, rhs);
    };
    auto pubkey_hash = hash(bytes_to_hash_type(take<32>(pubkey)), bytes_to_hash_type(take<32>(pubkey, 32)));
    #else
    auto hash = [](const HashType& lhs, const HashType& rhs) {
        return sha256(lhs, rhs);
    };
    auto pubkey_hash = sha256(pubkey);
    #endif

    // Hash branches.
    auto retval = hash(
        hash(hash(pubkey_hash, withdrawal_credentials), hash(effective_balance, slashed)),
        hash(hash(activation_eligibility_epoch, activation_epoch),
                  hash(exit_epoch, withdrawable_epoch)));

    return retval;
}

VoteToken verify_attestation_data_imp(const HashType& block_root, const Attestation& attestation, base_field_type sigma) {
    assert_true(sigma != 0);

    ssz_verify_proof(
        block_root, attestation.state_root, fill_zero_hashes(Proof(attestation.state_root_proof)).content(), 11, 3);

    ssz_verify_proof(attestation.state_root,
                     attestation.validators_root,
                     fill_zero_hashes(Proof(attestation.validators_root_proof)).content(),
                     43,
                     5);

    // We aggregate all validator pubkeys to verify the `signature`
    // field at the end.
    ///! aggregated_point = bls.Z1

    // Iterate over each validator that participates in this aggregated
    // attestation.

    base_field_type token = 0;

    for (size_t i = 0; i < attestation.validators.size(); i++) {
        auto& v = attestation.validators[i];
        // Aggregate this validator's public key.
        auto validator_pubkey = v.pubkey;
        ///! pubkey_point = pubkey_to_G1(validator_pubkey)
        ///! aggregated_point = bls.add(aggregated_point, pubkey_point)

        // Check if this validator was part of the source state.
        if (v.trusted) {
            auto leaf = hash_validator(circuit_byte_utils::expand<64>(v.pubkey),
                                       v.withdrawal_credentials,
                                       v.effective_balance,
                                       v.activation_eligibility_epoch,
                                       v.activation_epoch,
                                       v.exit_epoch,
                                       v.withdrawable_epoch);
            // Hash the validator data and make sure it's part of:
            // validators_root -> state_root -> block_root.
            ssz_verify_proof(attestation.validators_root,
                             leaf,
                             fill_zero_hashes(Proof(v.validator_list_proof)).content(),
                             0x020000000000ul + v.validator_index,
                             41);

            // TODO: Needed?
            // assert spec.is_active_validator(
            //     v['activation_epoch'],
            //     v['exit_epoch'],
            //     EPOCH,
            // )

            // Include this validator's pubkey in the result.
            base_field_type element;
            memcpy(&element, &(v.pubkey), sizeof(element));
            token = (token + (element * sigma));
        }
    }
    // Verify the aggregated signature.
    // aggregated_pubkey: BLSPubkey = G1_to_pubkey(aggregated_point)
    // signing_root: bytes = spec.compute_attestation_signing_root(
    //     attestation['fork'],
    //     attestation['genesis_validators_root'],
    //     attestation['data'],
    // )
    // signature: BLSSignature = BLSSignature(to_bytes(hexstr=attestation['signature']))
    // assert bls_pop.Verify(
    //     aggregated_pubkey,
    //     signing_root,
    //     signature,
    // )

    return VoteToken {{attestation.data.source, attestation.data.target}, token};
}

VoteToken combine_finality_votes(const static_vector<VoteToken, 8192>& tokens) {
    VoteToken result;
    result.transition = tokens[0].transition;
    result.token = {0};
    for (size_t i = 0; i < tokens.size(); i++) {
        assert_true(result.transition == tokens[i].transition);
        result.token += tokens[i].token;
    }
    return result;
}

uint64_t process_votes(const PubKey* trustedKeys,
                      const size_t pubkeysCount,
                      const int64_t sigma,
                      base_field_type& reconstructed_token) {
    const PubKey* prev = nullptr;
    uint64_t votes_count = 0;
    for (size_t i = 0; i < pubkeysCount; i++) {
        const auto& pubkey = trustedKeys[i];
        base_field_type element;
        memcpy(&element, &pubkey, sizeof(element));
        reconstructed_token = (reconstructed_token + element * sigma);
        if (prev && pubkey != *prev) {
            ++votes_count;
        }
        prev = &pubkey;
    }
    return votes_count;
}

void prove_finality(const VoteToken& token,
                    const PubKey* trustedKeys,
                    const size_t pubkeysCount,
                    const Transition& votedTransition,
                    const int64_t sigma,
                    const int64_t active_validators_count) {
    assert_true(votedTransition == token.transition);
    base_field_type reconstructed_token = 0;
    uint64_t votes_count = process_votes(trustedKeys, pubkeysCount, sigma, reconstructed_token);

    assert_true(reconstructed_token == token.token);
}
