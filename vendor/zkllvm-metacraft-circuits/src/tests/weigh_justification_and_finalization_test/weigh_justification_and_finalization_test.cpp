#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "circuit_utils/base_types.h"
#include "circuit_utils/circuit_byte_utils.h"
#include "circuit_utils/constants.h"

#include <algorithm>
#include <array>

#include <llvm/ObjectYAML/YAML.h>
#include <iostream>
#include <fstream>
#include <streambuf>

#include "utils/picosha2.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"

using llvm::yaml::Input;
using llvm::yaml::IO;
using llvm::yaml::MappingTraits;
using llvm::yaml::Output;

using namespace circuit_byte_utils;
using namespace file_utils;

using std::cout;

//#################################################################################################

Bytes32 sha256_pair(
    const Bytes32& left,
    const Bytes32& right
)
{
    Bytes32 ret_val{};
    Bytes64 combined{};
    std::copy(left.begin(), left.end(), combined.begin());
    std::copy(right.begin(), right.end(), combined.begin() + 32);

    picosha2::hash256(combined.begin(), combined.end(), ret_val.begin(), ret_val.end());

    return ret_val;
}

template <uint32_t MERKLE_DEPTH>
Bytes32 ssz_restore_merkle_root(
    const Bytes32& leaf,
    const std::array<Bytes32, MERKLE_DEPTH>& branch,
    uint64_t gindex
)
{
    auto hash = leaf;

    for(size_t i = 0; i < MERKLE_DEPTH; i++) {
        Bytes32 left;
        Bytes32 right;

        if(get_nth_bit(gindex, i) % 2 == 1) {
            left = branch[i];
            right = hash;
        } else {
            right = branch[i];
            left = hash;
        }
 
        std::array<unsigned char, 64> data{};
        size_t data_index = 0;
        for(size_t j = 0; j < 32; j++) {
            data[data_index++] = left[j];
        }
        for(size_t j = 0; j < 32; j++) {
            data[data_index++] = right[j];
        }

        picosha2::hash256(data.begin(), data.end(), hash.begin(), hash.end());
    }

    return hash;
}

template <uint32_t MERKLE_DEPTH>
void ssz_verify_proof(
    const Bytes32& root,
    const Bytes32& leaf,
    const std::array<Bytes32, MERKLE_DEPTH>& branch,
    const uint64_t gindex
)
{
    auto expected_root = ssz_restore_merkle_root<MERKLE_DEPTH>(leaf, branch, gindex);
    assert_true(root == expected_root);
}

Bytes32 hash_tree_root(uint64_t val)
{
    auto bytes = int_to_bytes(val);
    Bytes32 return_val{};
    std::copy(bytes.begin(), bytes.end(), return_val.begin());
    return return_val;
}

Bytes32 hash_tree_root(const CheckpointVariable& checkpoint)
{
    auto epoch_leaf = hash_tree_root(checkpoint.epoch);
    return sha256_pair(epoch_leaf, checkpoint.root);
}

Bytes32 hash_tree_root(const JustificationBitsVariable& checkpoint)
{
    Bytes32 ret_val{};
    for(auto i = 0; i < 4; i++) {
        if(checkpoint.bits[i]) {
            set_nth_bit(ret_val[0], i);
        }
    }

    return ret_val;
}

void verify_slot(
    const Root& beacon_state_root,
    const Slot& slot,
    const BeaconStateLeafProof& proof
)
{
    auto slot_leaf = hash_tree_root(slot);
    auto gindex = BEACON_STATE_SLOT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(beacon_state_root, slot_leaf, proof, gindex);
}

void verify_previous_justified_checkpoint(
    const Root& beacon_state_root,
    const CheckpointVariable& checkpoint,
    const BeaconStateLeafProof& proof
)
{
    const auto checkpoint_leaf = hash_tree_root(checkpoint);
    const auto gindex = BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(
        beacon_state_root, checkpoint_leaf, proof, gindex);
}

void verify_current_justified_checkpoint(
    Root beacon_state_root,
    CheckpointVariable checkpoint,
    BeaconStateLeafProof proof
)
{
    auto checkpoint_leaf = hash_tree_root(checkpoint);
    auto gindex = BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(
        beacon_state_root, checkpoint_leaf, proof, gindex);
}

void verify_justification_bits(
    Root beacon_state_root,
    JustificationBitsVariable justification_bits,
    BeaconStateLeafProof proof
)
{
    auto justification_bits_leaf = hash_tree_root(justification_bits);
    auto gindex = BEACON_STATE_JUSTIFICATION_BITS_GINDEX;
    ssz_verify_proof<array_size<BeaconStateLeafProof>::size>(
        beacon_state_root,
        justification_bits_leaf,
        proof,
        gindex
    );
}

//#################################################################################################

int main(int argc, char* argv[]) {

    uint64_t val = 191;
    auto val_bytes = hash_tree_root(val);
    std::cout << "bytesToHex(val_bytes) = " << byte_utils::bytesToHex(val_bytes) << "\n";

    Bytes32 left{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32};
    Bytes32 right{33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64};
    sha256_pair(left, right);

    std::cout << "array_size<BeaconStateLeafProof>::size = " << array_size<BeaconStateLeafProof>::size << "\n";

    JustificationBitsVariable jbv{};
    jbv.bits[1] = true;

    for(auto i = 0; i < jbv.bits.size(); i++) {
        std::cout << "jbv.bits[" << i << "] = " << jbv.bits[i] << "\n";
    }

    auto jbv_res = hash_tree_root(jbv);

    std::cout << "bytesToHex(jbv_res) = " << byte_utils::bytesToHex(jbv_res) << "\n";


    return 0;
}
