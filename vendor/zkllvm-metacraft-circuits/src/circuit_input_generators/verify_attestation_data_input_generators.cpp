
#include "circuit_utils/circuit_byte_utils.h"

#include <algorithm>
#include <array>
#include <cstring>

#include <iostream>
#include <fstream>
#include <streambuf>
#include <memory>

#include "utils/picosha2.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"
#include "serialization/serialize_attestation.h"

#include "circuits_impl/verify_attestation_data_imp.h"

using namespace circuit_byte_utils;
using namespace byte_utils;
using namespace file_utils;

using std::cout;

constexpr size_t MAX_KEYS = 1'000'000;

int main(int argc, char* argv[]) {

    if (0) {
        auto beacon_state_root = hexToBytes<32>("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

        std::cout << "byte_array_to_json(beacon_state_root) = " << byte_array_to_json(beacon_state_root) << "\n";

        auto my_json = byte_array_to_json(beacon_state_root);

        std::cout << "bytesToHex(json_to_byte_array<32>(byte_array_to_json(beacon_state_root))):\n"
                  << bytesToHex(json_to_byte_array<32>(byte_array_to_json(beacon_state_root))) << "\n";

        // std::cout << bytesToHex(json_to_byte_array<32>(nlohmann::json()));

        nlohmann::json the_result;

        the_result.push_back(byte_array_to_json(hexToBytes<32>("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a")));
        the_result.push_back(byte_array_to_json(hexToBytes<32>("96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1")));
        the_result.push_back(byte_array_to_json(hexToBytes<32>("ef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f")));
        the_result.push_back(byte_array_to_json(hexToBytes<32>("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7")));
        the_result.push_back(byte_array_to_json(hexToBytes<32>("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf")));

        // std::cout << "the_result: \n" << the_result.dump(2) << "\n";
        std::cout << "the_result: \n" << the_result << "\n";
    }

    std::cout << "src/circuit_input_generators/verify_attestation_data_input_generators.cpp \n";
    // path my_path("/finalizer-data/merged_234400.json");
    // std::ifstream f(my_path);
    // auto data = json::parse(f);

    // Transition voted_transition;
    // PubKey* trusted_pubkeys = (PubKey*)malloc(sizeof(PubKey) * 1'000'000);
    // size_t i = 0;
    // size_t unique_keys_count = 0;
    // for (auto& keys_set : data["trusted_pubkeys"]) {
    //     for (auto& keys : keys_set) {
    //         for (auto& key : keys) {
    //             if (i >= 2) {
    //                 std::string prev = "";
    //                 if (prev != std::string(key)) {
    //                     trusted_pubkeys[unique_keys_count++] = byte_utils::hexToBytes<48>(key);
    //                 }
    //                 prev = key;
    //             } else if (i == 0) {
    //                 voted_transition.source.epoch = key["epoch"];
    //                 voted_transition.source.root = byte_utils::hexToBytes<32>(key["root"]);
    //             } else if (i == 1) {
    //                 voted_transition.target.epoch = key["epoch"];
    //                 voted_transition.target.root = byte_utils::hexToBytes<32>(key["root"]);
    //             }
    //             ++i;
    //         }
    //     }
    // }

    // {
    //     nlohmann::json final_result;
    //     nlohmann::json keys_result;
    //     for (int i = 0; i < std::min(unique_keys_count, MAX_KEYS); ++i) {
    //         keys_result.push_back(byte_array_to_json(trusted_pubkeys[i]));
    //     }
    //     for(int i = 0; i < (int)MAX_KEYS - (int)unique_keys_count; ++i) {
    //         keys_result.push_back(byte_array_to_json(byte_utils::hexToBytes<48>("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")));
    //     }
    //     nlohmann::json array_keys_result;
    //     array_keys_result["array"] = keys_result;
    //     final_result.push_back(array_keys_result);
    //     // final_result.push_back(pack_int_json(unique_keys_count));
    //     // final_result.push_back(pack_int_json(0x69));

    //     // final_result.push_back(byte_array_to_json(trusted_pubkeys[0]));

    //     std::cout << final_result.dump(2) << "\n";

    // }

    AttestationData ad;
    ad.slot = 17;
    ad.index = 22;
    ad.beacon_block_root = hexToBytes<32>("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a");

    std::cout << "serialize<AttestationData>(ad):\n" << serialize<AttestationData>(ad).dump(2) << "\n";

    Validator v;
    v.trusted = true;
    v.validator_index = 910250;
    v.pubkey = hexToBytes<48>("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a1122334455667788991234567890abcd");
    v.withdrawal_credentials = hexToBytes<32>("232f5a216b76377c2618a186577d6ec88ab85c1507c01db2a58ffcb044a4a785");

    std::cout << "serialize<Validator>(v):\n" << serialize<Validator>(v).dump(2) << "\n";


    // free(trusted_pubkeys);

    return 0;
}
