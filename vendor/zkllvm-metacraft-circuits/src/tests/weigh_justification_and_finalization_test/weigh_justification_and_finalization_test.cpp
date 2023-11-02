#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "circuit_utils/circuit_byte_utils.h"

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
Bytes32 ssz_restore_merkle_root(
    const Bytes32& leaf,
    const std::array<Bytes32, MAX_MERKLE_DEPTH> branch,
    const unsigned int branch_depth,
    uint64_t gindex
)
{
    auto hash = leaf;

    for(size_t i = 0; i < branch_depth; i++) {
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

void ssz_verify_proof(
    const Bytes32 root,
    const Bytes32 leaf,
    const std::array<Bytes32, MAX_MERKLE_DEPTH> branch,
    const unsigned int branch_depth,
    const uint64_t gindex
) {
    auto expected_root = ssz_restore_merkle_root(leaf, branch, branch_depth, gindex);
    assert_true(root == expected_root);
}

void verify_slot(
    Root beacon_state_root,
    Slot slot,
    BeaconStateLeafProof proof
) {
    // auto slot_leaf = slot.hash_tree_root(builder);
    // auto gindex = U64Variable::constant(builder, BEACON_STATE_SLOT_GINDEX);
    // ssz_verify_proof(beacon_state_root, slot_leaf, proof.as_slice(), gindex);
}

//#################################################################################################

int main(int argc, char* argv[]) {



    return 0;
}
