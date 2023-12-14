#pragma once

#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/algebra/curves/pallas.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "../circuit_utils/circuit_byte_utils.h"
#include "../circuit_utils/ssz_utils.h"
#include "../utils/picosha2.h"
#include "../circuit_utils/static_vector.h"

using namespace circuit_byte_utils;
using namespace ssz_utils;
using namespace nil::crypto3::algebra::curves;

using Proof = static_vector<Bytes32>;
using PubKey = Bytes48;

struct AttestationData {
    int64_t slot;
    int64_t index;
    Root beacon_block_root;
    CheckpointVariable source;
    CheckpointVariable target;
};

struct Fork {
    Bytes32 previous_version;
    Bytes32 current_version;
    int64_t epoch;
};

struct Validator {
    // True if it can be proven that this validator is a member of the
    // source state and can, therefore, be trusted.
    bool trusted;
    // These fields are always present.
    int64_t validator_index;
    PubKey pubkey;
    // These fields are meaningful iff `trusted` is True.
    Bytes32 withdrawal_credentials;
    int64_t effective_balance;
    bool slashed;
    int64_t activation_eligibility_epoch;
    int64_t activation_epoch;
    int64_t exit_epoch;
    int64_t withdrawable_epoch;
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
    bool operator==(const Transition &c) const {
        return (source == c.source && target == c.target);
    }
};

struct TransitionKeys {
    Transition transition;
    static_vector<Bytes32> keys;
};

struct Merged {
    static_vector<Attestation> attestations;
    static_vector<TransitionKeys> trusted_pubkeys;
};

using base_field_type = typename pallas::base_field_type::value_type;

struct VoteToken {
    Transition transition;
    base_field_type token;
};

using TransitionKey = Bytes32;

static_vector<Bytes32> compute_zero_hashes(int length = 64) {
    static_vector<Bytes32> xs;
    xs.push_back(get_empty_byte_array<32>());
    for(int i = 1; i < length; i++) {
        xs.push_back(sha256(xs[i-1], xs[i-1]));
    }
    return xs;
}

static const auto zero_hashes = compute_zero_hashes();

static const auto empty_hash = get_empty_byte_array<32>();

static_vector<Bytes32> fill_zero_hashes(
        const static_vector<Bytes32>& xs,
        size_t length = 0
)
{
    static_vector<Bytes32> ws = xs;
    int additions_count = length - xs.size();

    for(int i = 0; i < ws.size(); i++) {
        if(ws[i] == empty_hash) {
            ws[i] = zero_hashes[i];
        }
    }
    for(int i = additions_count; i > 0; i--) {
        ws.push_back(zero_hashes[xs.size() + additions_count - i]);
    }
    return ws;
}

Bytes32 hash_validator(
    Bytes64 pubkey,
    Bytes32 withdrawal_credentials,
    uint64_t effective_balance_,
    uint64_t activation_eligibility_epoch_,
    uint64_t activation_epoch_,
    uint64_t exit_epoch_,
    uint64_t withdrawable_epoch_
)
{
    // Convert parameters.
    auto effective_balance = int_to_bytes<uint64_t, 32, true>(effective_balance_);
    auto slashed = int_to_bytes<uint64_t, 32, true>(0);
    auto activation_eligibility_epoch = int_to_bytes<uint64_t, 32, true>(activation_eligibility_epoch_);
    auto activation_epoch = int_to_bytes<uint64_t, 32, true>(activation_epoch_);
    auto exit_epoch = int_to_bytes<uint64_t, 32, true>(exit_epoch_);
    auto withdrawable_epoch = int_to_bytes<uint64_t, 32, true>(withdrawable_epoch_);

    // Hash branches.
    auto retval = sha256(
        sha256(
            sha256(sha256(pubkey)           , withdrawal_credentials),
            sha256(effective_balance           , slashed)
        ),
        sha256(
            sha256(activation_eligibility_epoch, activation_epoch),
            sha256(exit_epoch                  , withdrawable_epoch)
        )
    );
    return retval;
}

VoteToken verify_attestation_data(Bytes32 block_root, Attestation attestation, base_field_type sigma) {
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

    // We aggregate all validator pubkeys to verify the `signature`
    // field at the end.
    ///! aggregated_point = bls.Z1

    // Iterate over each validator that participates in this aggregated
    // attestation.

    base_field_type token = 0;

    for (auto v = attestation.validators.begin(); v != attestation.validators.end(); v++) {
        // Aggregate this validator's public key.
        auto validator_pubkey = v->pubkey;
        ///! pubkey_point = pubkey_to_G1(validator_pubkey)
        ///! aggregated_point = bls.add(aggregated_point, pubkey_point)

        // Check if this validator was part of the source state.
        if (v->trusted) {
            auto leaf = hash_validator(
                circuit_byte_utils::expand<64>(v->pubkey),
                v->withdrawal_credentials,
                v->effective_balance,
                v->activation_eligibility_epoch,
                v->activation_epoch,
                v->exit_epoch,
                v->withdrawable_epoch
            );
            // Hash the validator data and make sure it's part of:
            // validators_root -> state_root -> block_root.
            ssz_verify_proof(
                attestation.validators_root,
                leaf,
                fill_zero_hashes(v->validator_list_proof).content(),
                0x020000000000ul + v->validator_index,
                41
            );

            // TODO: Needed?
            // assert spec.is_active_validator(
            //     v['activation_epoch'],
            //     v['exit_epoch'],
            //     EPOCH,
            // )

            // Include this validator's pubkey in the result.
            base_field_type element; 
            memcpy(&element, &(v->pubkey), sizeof(element));
            token = (token + (element * sigma) );
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

    return VoteToken {
        { attestation.data.source,
          attestation.data.target },
        token
    };
}

/*
def combine_finality_votes(tokens: Iterable[VoteToken]) -> Mapping[TransitionKey, int]:
    f: Callable[[int, int], int] = lambda memo, x: (memo + x) % MODULUS
    g: Callable[[Iterator[VoteToken]], int] = lambda xs: reduce(f, [x[1] for x in xs], 0)
    grouped: groupby[Transition, VoteToken] = groupby(tokens, key=itemgetter(0))
    return {
        transition_to_key(transition): g(group)
        for (transition, group) in grouped
    }
*/

VoteToken combine_finality_votes(static_vector<VoteToken, 8192> tokens)
{
    VoteToken result;
    result.transition = tokens[0].transition;
    result.token = 0;
    for(auto it = tokens.begin(); it != tokens.end(); it++) {
        assert_true(result.transition == it->transition);
        result.token += it->token;
    }
    return result;
}

void prove_finality(
        VoteToken token,
        static_vector<PubKey> trustedKeys,
        Transition votedTransition,
        int sigma,
        int64_t active_validators_count
)
{
    assert_true(votedTransition == token.transition);
    int64_t votes_count = 0;
    PubKey* prev = nullptr;
    base_field_type reconstructed_token = 0;
    for(auto it = trustedKeys.begin(); it != trustedKeys.end(); it++) {
        auto& pubkey = *it;
        base_field_type element;
        memcpy(&element, &pubkey, sizeof(element));
        reconstructed_token = (reconstructed_token + element*sigma);
        if(prev && pubkey != *prev) {
            votes_count++;
        }
        prev = &pubkey;
    }
    assert_true(reconstructed_token == token.token);
}
   