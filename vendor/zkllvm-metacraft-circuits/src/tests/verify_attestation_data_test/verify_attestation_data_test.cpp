#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/algebra/curves/pallas.hpp>

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

#include "json/json.hpp"
using namespace nlohmann;


#include "circuits_impl/verify_attestation_data_imp.h"

using namespace nil::crypto3::algebra::curves;
using llvm::yaml::Input;
using llvm::yaml::IO;
using llvm::yaml::MappingTraits;
using llvm::yaml::Output;

using namespace circuit_byte_utils;
using namespace file_utils;

using std::cout;

int main(int argc, char* argv[]) {

    static_vector<Bytes32> hashes;

    hashes.push_back(byte_utils::hexToBytes<32>("0x0000000000000000000000000000000000000000000000000000000000000000"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x12343211234120302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x111111111111d4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x222222222222c56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x333333333333d165a55d5eeae91485954472d56f246df256bf3cae19352a123c"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x444444444444429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30"));
    hashes.push_back(byte_utils::hexToBytes<32>("0x5555555555555555555555555555555555555555555555555555555555555555"));

    auto modified = fill_zero_hashes(hashes, 2);

    for(auto it = modified.begin(); it != modified.end(); it++) {
        std::cout << byte_utils::bytesToHex(*it) << "\n";
    }
    {
        auto hashed_validator = hash_validator(
            circuit_byte_utils::expand<64>(
                byte_utils::hexToBytes<48>(
                    "8dac0b1f39066e1c902dfe24f45bc473e8894959ad8da765a447c108fe754ab07a4eeec1e59dea3ef961bf190c22ad82")),
            byte_utils::hexToBytes<32>("01000000000000000000000061fa6204b232b3e8f3eb388b50a2f2574c9052a6"),
            32000000000ul,
            226977ul,
            230998ul,
            18446744073709551615ul,
            18446744073709551615ul
        );

        printf("Expected:   40f8fcd65d42c86a6ad0ac9dea4ca6fa83364f500f11a748d18b158e2e3e6594\n");
        printf("Calculated: %s\n", byte_utils::bytesToHex(hashed_validator).c_str());
    }
    {
        auto hashed_validator = hash_validator(
            circuit_byte_utils::expand<64>(
                byte_utils::hexToBytes<48>(
                    "a601a72aeb69888c426dae588ee0ef79cb7d3a1389d6955a4b979cea48a069068b230d733cb0a47db2b1db2cd517ca28")),
            byte_utils::hexToBytes<32>("005235facd5c0beff85310b0aadf7306c9f11c0d92af36530f1c51e84ee0526b"),
            32000000000ul,
            148259ul,
            148274ul,
            18446744073709551615ul,
            18446744073709551615ul
        );

        printf("Expected:   496b1e4562f133ebad777d05695cab85835052243a931d91e6d59d69241d309e\n");
        printf("Calculated: %s\n", byte_utils::bytesToHex(hashed_validator).c_str());
    }
    {
        auto hashed_validator = hash_validator(
            circuit_byte_utils::expand<64>(
                byte_utils::hexToBytes<48>(
                    "87cbc98ab8a333c199fbf5ba562083e823b48a0e411dfc7492f039e863b6d68764fed36ca1efa1a46b5a779055b46468")),
            byte_utils::hexToBytes<32>("010000000000000000000000e839a3e9efb32c6a56ab7128e51056585275506c"),
            32000000000ul,
            200484ul,
            204972ul,
            18446744073709551615ul,
            18446744073709551615ul
        );

        printf("Expected:   c5f5ad3d3adb399b15b1d1513207e9c5d4cdb7234019a62fa0774ef3f67772e3\n");
        printf("Calculated: %s\n", byte_utils::bytesToHex(hashed_validator).c_str());
    }
    {
        static_vector<Bytes32> proof;
        proof.push_back(byte_utils::hexToBytes<32>("6823ef320178e99dbb3437e283fb8ec25b870c2ceac62d3db549ca21c9cd7ec4"));
        proof.push_back(byte_utils::hexToBytes<32>("38864832a7e2e17e177cf615cf04ed5a4c9f3b10e9bc271fd1c4a3efe2a78529"));
        proof.push_back(byte_utils::hexToBytes<32>("e8baa1746a3bbab8401b41a9e20a289c582b8c3a04dc5e25aba00e6d37cb1b64"));
        proof.push_back(byte_utils::hexToBytes<32>("4d2c3cb4a377efe1d037c98b78ea94f716402d87bf8d3ec85e52530517dd245d"));
        proof.push_back(byte_utils::hexToBytes<32>("5698570e5020fb13b3031804bab9b84d782dc97d0717aa396e977c24ae23983a"));
        proof.push_back(byte_utils::hexToBytes<32>("d62852c649334800b50574e6e032505b057373ca5ae9618e325fe7055a124fde"));
        proof.push_back(byte_utils::hexToBytes<32>("f6fc7745dc0639dfdd9ee802b845824dcfe3a7214426ffa6326b1e0e3fd16164"));
        proof.push_back(byte_utils::hexToBytes<32>("630cdc3baaa425eb101be81876a145f8150829e7c042b4ef206fea0e6ea85f14"));
        proof.push_back(byte_utils::hexToBytes<32>("7a16d7eecc969b7dce42d09308948caba1d7c2f063e302f6b49beab9c9cfa6ae"));
        proof.push_back(byte_utils::hexToBytes<32>("ec4c4c11c6f26b58dc55760b89ec6d3f7d3707bebf0169aff6d0e3e2a3061124"));
        proof.push_back(byte_utils::hexToBytes<32>("f632776d25dcfe5af6c50da113bb0226ab5b4d23710858bc22f8d1dc5b31a8a5"));
        proof.push_back(byte_utils::hexToBytes<32>("75a5d5657cec549fa77416e1f34826760a146d0f8eb8bea9e88c1b746c2669af"));
        proof.push_back(byte_utils::hexToBytes<32>("baf96aaafe03e7f74ef7b435e646c0017e93401a8dd6a407e970adc85f4ae13e"));
        proof.push_back(byte_utils::hexToBytes<32>("d24c481836d9ea3e9d929cd4a29254c571eeba95783326a8518184845a1a8e7a"));
        proof.push_back(byte_utils::hexToBytes<32>("2e9574f1d717388fdf894b18403b7312016789404520f4038c1628cccca785fc"));
        proof.push_back(byte_utils::hexToBytes<32>("735388d283314aabe82fd66a896138b7122d8e695453ffadcd7d2e4fffbd81c4"));
        proof.push_back(byte_utils::hexToBytes<32>("e8a2578efdd81b84840886cf1ed6eca7837843404236d55271deca7fac9fab79"));
        proof.push_back(byte_utils::hexToBytes<32>("4f836bbdb1d3bea778141dcfb130425f538a4f2b5d2b4b3e243c0fcab0afba4a"));
        proof.push_back(byte_utils::hexToBytes<32>("e0047ed3ca8e8420ba23bed3c866b72e6404c7661d73d5ddc65e401a8344dee8"));
		proof.push_back(byte_utils::hexToBytes<32>("9e83394871b3b699f1bc6bbf83bc657fb2dfbeca2db7758812a9261189e0ba23"));
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000")); 
        proof.push_back(byte_utils::hexToBytes<32>("0000000000000000000000000000000000000000000000000000000000000000"));
		proof.push_back(byte_utils::hexToBytes<32>("6bab0e0000000000000000000000000000000000000000000000000000000000"));

        auto res123 = fill_zero_hashes(proof);
        for(auto it = res123.begin(); it != res123.end(); it++) {
            std::cout << byte_utils::bytesToHex(*it) << "\n";
        }

        printf("ssz_verify_proof ... ");
        ssz_verify_proof(
            byte_utils::hexToBytes<32>("b45a79b3d4ed0bce770893498237fafc26885ca1a23a1e77933de33c02802db5"),
            byte_utils::hexToBytes<32>("64df3a60d06291506b1e0de11ce4bac1e1cd0e2e3f639d786128c9b79475a78c"),
            fill_zero_hashes(proof).content(),
            0x020000000000ul + 818904,
            41
        );
        printf("Done\n");


        std::array<unsigned char, 32> key;
        typename pallas::base_field_type::value_type pkey;

        static_assert(sizeof(pkey) >= sizeof(key));

        memcpy(&pkey, &key, sizeof(key));

        std::cout << "sizeof(pkey) = " << sizeof(pkey) << "\n";

    }

    // print(f"uint_to_b32(1234512345) = {uint_to_b32(1234512345)}")
    // print(f"bytes_to_u64(var) = {bytes_to_u64(var)}")
    using namespace byte_utils;
    Bytes32 val = int_to_bytes<uint64_t, 32, true>(1234512345);
    std::cout << bytesToHex(val) << "\n";
    std::cout << bytes_to_int<uint64_t, 32>(val) << "\n";

    path my_path("/finalizer-data/merged_234400.json");
    std::ifstream f(my_path);
    auto data = json::parse(f);

    // Yep, this was value was chosen randomly.
    base_field_type sigma = 0x69;

    // Run the first circuit for each attestation.
    /*
    tokens: List[VoteToken] = []
    n: int = len(inp['attestations'])
    for i, attestation in enumerate(inp['attestations']):
        print(f'Processing attestation {i}/{n}...')
        vote: VoteToken = circuits.verify_attestation_data(
            'd5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7',
            attestation,
            sigma,
        )
        tokens.append(vote)
    with open(BASE_DIR / 'tests/cache.json', 'wb') as f:
        f.write(TypeAdapter(List[VoteToken]).dump_json(tokens))
    */
    // for(const auto& attestation : data["attestations"]) {

    // }

    return 0;
}
