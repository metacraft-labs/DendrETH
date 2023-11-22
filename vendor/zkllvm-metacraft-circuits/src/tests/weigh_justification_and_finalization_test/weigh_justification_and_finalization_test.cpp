#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <llvm/ObjectYAML/YAML.h>
#include "json/json.hpp"
#include <iostream>
#include <fstream>
#include <streambuf>

#include "circuits_impl/weigh_justification_and_finalization.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"

using llvm::yaml::Input;
using llvm::yaml::IO;
using llvm::yaml::MappingTraits;
using llvm::yaml::Output;

using namespace byte_utils;
using namespace file_utils;
using namespace weigh_justification_and_finalization_;

using std::cout;

std::ostream& operator<<(std::ostream& os, const CheckpointVariable& c) {
    os << "CheckpointValue { epoch: " << c.epoch << ", root: " << bytesToHex(c.root) << " }";
    return os;
}

std::ostream& operator<<(std::ostream& os, const JustificationBitsVariable& j) {
    os << " [" << (j.bits[0] ? "true, " : "false, ") << (j.bits[1] ? "true, " : "false, ")
       << (j.bits[2] ? "true, " : "false, ") << (j.bits[3] ? "true]" : "false]");
    return os;
}

void test_circuit_sample_data() {

    auto beacon_state_root =
        hexToBytes<32>("0x87a7acf1710775a4f1c1604477e4d2b5f111e06b184c8e447c2c573346520672");

    auto slot = 6953401;

    BeaconStateLeafProof slot_proof {
        hexToBytes<32>("b85c1507c01db2a58ffcb044a4a785232f5a216b76377c2618a186577d6ec88a"),
        hexToBytes<32>("96a9cb37455ee3201aed37c6bd0598f07984571e5f0593c99941cb50af942cb1"),
        hexToBytes<32>("ef23aac89399ee4e947be08261ad233800add160fc7f5b86bff0d94a061a404f"),
        hexToBytes<32>("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        hexToBytes<32>("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    CheckpointVariable previous_justified_checkpoint {
        217291,
        hexToBytes<32>("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    BeaconStateLeafProof previous_justified_checkpoint_proof {
        hexToBytes<32>("0xf7b1fc5e9ef34f7455c8cc475a93eccc5cd05a3879e983a2bad46bbcbb2c71f5"),
        hexToBytes<32>("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    CheckpointVariable current_justified_checkpoint {
        217292,
        hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1"),
    };

    BeaconStateLeafProof current_justified_checkpoint_proof {
        hexToBytes<32>("0x2b913be7c761bbb483a1321ff90ad13669cbc422c8e23eccf9eb0137c8c3cf48"),
        hexToBytes<32>("0xedaaa63d1f9e2e4564ce78f62dc7130511d2edf70d76c3106be94da93fb8594a"),
        hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    JustificationBitsVariable justification_bits {true, true, true, true};

    BeaconStateLeafProof justification_bits_proof {
        hexToBytes<32>("0x1fca1f5d922549df42d4b5ca272bd4d022a77d520a201d5f24739b93f580a4e0"),
        hexToBytes<32>("0x9f1e3e59c7a4606e788c4e546a573a07c6c2e66ebd245aba2ff966b27e8c2d4f"),
        hexToBytes<32>("0xcaac4c42893341c15c557df194682f42b6037a99fcec7d581d7624f470f05c06"),
        hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    MerkleProof<18> previous_epoch_start_slot_root_in_block_roots_proof {
        hexToBytes<32>("0x73dea1035b1bd431ccd1eaa893ad5f4b8488e68d2ca90615e5be0d8f7ba5a650"),
        hexToBytes<32>("0x0f7c6aa59235e573a4cdfb9411d5e4eb6255571814906c5928c016626aa2ff0a"),
        hexToBytes<32>("0xf770f73c2e01ddf6c71765e327eebb7bab0ab13f4506c736dfd6556037c0e646"),
        hexToBytes<32>("0x036f0750c86fdc21edee72b6ac1b5f728eed354c99d3b6862adf60f72bc5dcbe"),
        hexToBytes<32>("0x9730c8f3978ea7a1797603b19514e74273898f2be969ca8c583f55d14cd08d03"),
        hexToBytes<32>("0x47b601e8c14026380bdd0f716a4188e9f50a86bc58f0c342ead2a075ba8e5bc0"),
        hexToBytes<32>("0x6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        hexToBytes<32>("0x82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        hexToBytes<32>("0x30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        hexToBytes<32>("0xc9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        hexToBytes<32>("0x606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        hexToBytes<32>("0x4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        hexToBytes<32>("0xf3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        hexToBytes<32>("0xc524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        hexToBytes<32>("0xe3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        hexToBytes<32>("0x844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        hexToBytes<32>("0x2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        hexToBytes<32>("0x71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    MerkleProof<18> current_epoch_start_slot_root_in_block_roots_proof {
        hexToBytes<32>("c798192e5a066fe1ff3fc632bccd30a1ff47dc4d36909725db43ca6b23a5a7ba"),
        hexToBytes<32>("3161f17c79044792fc7c965a3fcb105f595bf895a44a774b871fa3017f5a36cc"),
        hexToBytes<32>("e3dddf89fa44413c3d4cf1762d7500b169116125194d96e86257cb616949560f"),
        hexToBytes<32>("3bfbdebbb29b9e066e08350d74f66116b0221c7d2c98724288a8e02bc7f937ae"),
        hexToBytes<32>("f50adbe1bff113f5d5535eee3687ac3b554af1eb56f8c966e572f8db3a61add3"),
        hexToBytes<32>("1a973e9b4fc1f60aea6d1453fe3418805a71fd6043f27a1c32a28bfcb67dd0eb"),
        hexToBytes<32>("6c006d77badeb95adb44e947b4feb8280757a94ce80033c092a108554bc830e7"),
        hexToBytes<32>("82f9367d0fceb05f6ece224c4dfae0aeb907bb11e1296a25bf6d82df79927e35"),
        hexToBytes<32>("30c8368deeb92bd11f57c58969331e2e815ce537f100e51d9168f4077d676b0e"),
        hexToBytes<32>("c9dc885d80ae7fbe70ab020bee374480efa9333ee540125f1334dde0ecd0abb0"),
        hexToBytes<32>("606d5714c14e3c4d92245cd4def7a72ed94433fef7d4d2b3971ce9d6d68fb4b5"),
        hexToBytes<32>("4fd67a88677841d71d2887b629d341e7007fdc0f8d35220a1c8b738c7390dab9"),
        hexToBytes<32>("f3e8e14e29e2b8a3ecf0579104b9795db5ae8c27c85c0e23421fec6193309a09"),
        hexToBytes<32>("c524bb3c09211599514c4146b1f109551ccce70826f359f87ce780c177160a1a"),
        hexToBytes<32>("e3b723a252e9ca9f904a74143a31b8d0115df6db02f74f0fc992926c80edd641"),
        hexToBytes<32>("844ee47d27dcc46ccbcdda65c8ab3dcdae34a7eff6ce4ab77cb8c78c36e72819"),
        hexToBytes<32>("2dd00c742aa4b987fe238286414c22b8d85b5caa469f3c35431693cbe46631d7"),
        hexToBytes<32>("71f820aab5b9e7848c94dad326e5badf4b3ccd7865a752c8e90bc68d8c5a05bf"),
    };

    auto previous_epoch_start_slot_root_in_block_roots =
        hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1");
    auto current_epoch_start_slot_root_in_block_roots =
        hexToBytes<32>("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c");

    auto total_active_balance = 10;
    auto previous_epoch_target_balance = 10;
    auto current_epoch_target_balance = 20;

    CheckpointVariable finalized_checkpoint {
        217291,
        hexToBytes<32>("0xf6e7dd9931e1e8d792908e5c6014778d252357f4e8942920a866dd6675626104"),
    };

    BeaconStateLeafProof finalized_checkpoint_proof {
        hexToBytes<32>("0x26803d08d4a1a3d223ed199292fa78e62ef586391213548388375f302acfdece"),
        hexToBytes<32>("0xf0af1bff0357d4eb3b97bd6f7310a71acaff5c1c1d9dde7f20295b2002feccaf"),
        hexToBytes<32>("0x43e892858dc13eaceecec6b690cf33b7b85218aa197eb1db33de6bea3d3374c2"),
        hexToBytes<32>("0x18d01635cb93bbf01263b79b3de8302211264ab2f3a3e0833f77e508a1abaaa1"),
        hexToBytes<32>("0x938c96912b5c4683b27fa6edc5d8b76ceb31d3c4ffce919382f59ba3ed3a079f"),
    };

    CheckpointVariable new_previous_justified_checkpoint;
    CheckpointVariable new_current_justified_checkpoint;
    CheckpointVariable new_finalized_checkpoint;
    JustificationBitsVariable new_justification_bits;

    weigh_justification_and_finalization_impl(beacon_state_root,
                                         slot,
                                         slot_proof,
                                         previous_justified_checkpoint,
                                         previous_justified_checkpoint_proof,
                                         current_justified_checkpoint,
                                         current_justified_checkpoint_proof,
                                         justification_bits,
                                         justification_bits_proof,
                                         total_active_balance,
                                         previous_epoch_target_balance,
                                         current_epoch_target_balance,
                                         previous_epoch_start_slot_root_in_block_roots,
                                         previous_epoch_start_slot_root_in_block_roots_proof,
                                         current_epoch_start_slot_root_in_block_roots,
                                         current_epoch_start_slot_root_in_block_roots_proof,
                                         finalized_checkpoint,
                                         finalized_checkpoint_proof,
                                         // Outputs:
                                         new_previous_justified_checkpoint,
                                         new_current_justified_checkpoint,
                                         new_finalized_checkpoint,
                                         new_justification_bits);

    std::cout << "outputs:\n";
    std::cout << "new_previous_justified_checkpoint: " << new_previous_justified_checkpoint << "\n";
    std::cout << "new_current_justified_checkpoint: " << new_current_justified_checkpoint << "\n";
    std::cout << "new_finalized_checkpoint: " << new_finalized_checkpoint << "\n";
    std::cout << "new_justification_bits: " << new_justification_bits << "\n";

    const CheckpointVariable expected_new_previous_justified_checkpoint {
        217292, hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1")};
    const CheckpointVariable expected_new_current_justified_checkpoint {
        217293, hexToBytes<32>("0x386f84f9d82ec2e8ae6ff584ef7f62f07da47f0163a3b9ce6f263107ac6e1c9c")};
    const CheckpointVariable expected_new_finalized_checkpoint {
        217292, hexToBytes<32>("0xc014dab4e45229aa677898bac663fe791c2d4ec62af0e328f02c5a0ba3f1eeb1")};
    const JustificationBitsVariable expected_new_justification_bits {true, true, true, true};

    std::cout << "expected outputs:\n";
    std::cout << "new_previous_justified_checkpoint: " << expected_new_previous_justified_checkpoint << "\n";
    std::cout << "new_current_justified_checkpoint: " << expected_new_current_justified_checkpoint << "\n";
    std::cout << "new_finalized_checkpoint: " << expected_new_finalized_checkpoint << "\n";
    std::cout << "new_justification_bits: " << expected_new_justification_bits << "\n";

    assert_true(expected_new_previous_justified_checkpoint == new_previous_justified_checkpoint);
    assert_true(expected_new_current_justified_checkpoint == new_current_justified_checkpoint);
    assert_true(expected_new_finalized_checkpoint == new_finalized_checkpoint);
    assert_true(expected_new_justification_bits == new_justification_bits);
}

void test_circuit_ssz_json() {

    using namespace nlohmann;

    std::vector<path> result;
    path my_path("./consensus-spec-tests");
    try {
        find_matching_files(my_path, std::vector<std::string> {"ssz_snappy.json"}, result);
    } catch (const non_existing_path& e) {
        std::cerr << "ERROR: non existing path " << e.what() << "\n";
        exit(1);
    }

    for (const auto& v : result) {
        std::cout << "processing file: " << v << " ...\n";

        std::ifstream f(v);
        json data = json::parse(f);
        std::cout << "slot = " << data["slot"] << "\n";
        std::cout << "slot_proof = " << data["slot_proof"] << "\n";
        std::cout << "previous_justified_checkpoint = " << data["previous_justified_checkpoint"] << "\n";
        std::cout << "previous_justified_checkpoint_proof = " << data["previous_justified_checkpoint_proof"] << "\n";
        std::cout << "current_justified_checkpoint = " << data["current_justified_checkpoint"] << "\n";
        std::cout << "current_justified_checkpoint_proof = " << data["current_justified_checkpoint_proof"] << "\n";
        std::cout << "justification_bits = " << data["justification_bits"] << "\n";
        std::cout << "justification_bits_proof = " << data["justification_bits_proof"] << "\n";
        std::cout << "previous_epoch_start_slot_root_in_block_roots = " << data["previous_epoch_start_slot_root_in_block_roots"] << "\n";
        std::cout << "previous_epoch_start_slot_root_in_block_roots_proof = " << data["previous_epoch_start_slot_root_in_block_roots_proof"] << "\n";
        std::cout << "current_epoch_start_slot_root_in_block_roots = " << data["current_epoch_start_slot_root_in_block_roots"] << "\n";
        std::cout << "current_epoch_start_slot_root_in_block_roots_proof = " << data["current_epoch_start_slot_root_in_block_roots_proof"] << "\n";
        std::cout << "finalized_checkpoint = " << data["finalized_checkpoint"] << "\n";
        std::cout << "finalized_checkpoint_proof = " << data["finalized_checkpoint_proof"] << "\n";
        std::cout << "state_root = " << data["beacon_state_root"] << "\n";

        std::stringstream buff;
        Slot slot { stringToUint64(data["slot"]) }; 
        BeaconStateLeafProof slot_proof {
            hexToBytes<32>(data["slot_proof"][0]),
            hexToBytes<32>(data["slot_proof"][1]),
            hexToBytes<32>(data["slot_proof"][2]),
            hexToBytes<32>(data["slot_proof"][3]),
            hexToBytes<32>(data["slot_proof"][4])
        };
        CheckpointVariable previous_justified_checkpoint {
            stringToUint64(data["previous_justified_checkpoint"]["epoch"]),
            hexToBytes<32>(data["previous_justified_checkpoint"]["root"])
        };
        BeaconStateLeafProof previous_justified_checkpoint_proof {
            hexToBytes<32>(data["previous_justified_checkpoint_proof"][0]),
            hexToBytes<32>(data["previous_justified_checkpoint_proof"][1]),
            hexToBytes<32>(data["previous_justified_checkpoint_proof"][2]),
            hexToBytes<32>(data["previous_justified_checkpoint_proof"][3]),
            hexToBytes<32>(data["previous_justified_checkpoint_proof"][4])
        };
        CheckpointVariable current_justified_checkpoint {
            stringToUint64(data["current_justified_checkpoint"]["epoch"]),
            hexToBytes<32>(data["current_justified_checkpoint"]["root"])
        };
        BeaconStateLeafProof current_justified_checkpoint_proof {
            hexToBytes<32>(data["current_justified_checkpoint_proof"][0]),
            hexToBytes<32>(data["current_justified_checkpoint_proof"][1]),
            hexToBytes<32>(data["current_justified_checkpoint_proof"][2]),
            hexToBytes<32>(data["current_justified_checkpoint_proof"][3]),
            hexToBytes<32>(data["current_justified_checkpoint_proof"][4])
        };

        JustificationBitsVariable justification_bits = hexToBitsVariable(data["justification_bits"]);

        BeaconStateLeafProof justification_bits_proof {
            hexToBytes<32>(data["justification_bits_proof"][0]),
            hexToBytes<32>(data["justification_bits_proof"][1]),
            hexToBytes<32>(data["justification_bits_proof"][2]),
            hexToBytes<32>(data["justification_bits_proof"][3]),
            hexToBytes<32>(data["justification_bits_proof"][4])
        };

        auto previous_epoch_start_slot_root_in_block_roots = hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots"]);
        MerkleProof<18> previous_epoch_start_slot_root_in_block_roots_proof {
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][0]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][1]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][2]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][3]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][4]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][5]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][6]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][7]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][8]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][9]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][10]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][11]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][12]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][13]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][14]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][15]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][16]),
            hexToBytes<32>(data["previous_epoch_start_slot_root_in_block_roots_proof"][17])
        };

        auto current_epoch_start_slot_root_in_block_roots = hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots"]);
        MerkleProof<18> current_epoch_start_slot_root_in_block_roots_proof {
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][0]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][1]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][2]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][3]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][4]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][5]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][6]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][7]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][8]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][9]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][10]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][11]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][12]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][13]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][14]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][15]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][16]),
             hexToBytes<32>(data["current_epoch_start_slot_root_in_block_roots_proof"][17])
        };
        CheckpointVariable finalized_checkpoint {
            stringToUint64(data["finalized_checkpoint"]["epoch"]),
            hexToBytes<32>(data["finalized_checkpoint"]["root"])
        };
        BeaconStateLeafProof finalized_checkpoint_proof {
            hexToBytes<32>(data["finalized_checkpoint_proof"][0]),
            hexToBytes<32>(data["finalized_checkpoint_proof"][1]),
            hexToBytes<32>(data["finalized_checkpoint_proof"][2]),
            hexToBytes<32>(data["finalized_checkpoint_proof"][3]),
            hexToBytes<32>(data["finalized_checkpoint_proof"][4])
        };
        auto beacon_state_root = hexToBytes<32>(data["beacon_state_root"]);

        CheckpointVariable new_previous_justified_checkpoint;
        CheckpointVariable new_current_justified_checkpoint;
        CheckpointVariable new_finalized_checkpoint;
        JustificationBitsVariable new_justification_bits;

        uint64_t total_active_balance=0,
                 previous_epoch_target_balance=0,
                 current_epoch_target_balance=0;

        weigh_justification_and_finalization_impl(beacon_state_root,
                                             slot,
                                             slot_proof,
                                             previous_justified_checkpoint,
                                             previous_justified_checkpoint_proof,
                                             current_justified_checkpoint,
                                             current_justified_checkpoint_proof,
                                             justification_bits,
                                             justification_bits_proof,
                                             total_active_balance,
                                             previous_epoch_target_balance,
                                             current_epoch_target_balance,
                                             previous_epoch_start_slot_root_in_block_roots,
                                             previous_epoch_start_slot_root_in_block_roots_proof,
                                             current_epoch_start_slot_root_in_block_roots,
                                             current_epoch_start_slot_root_in_block_roots_proof,
                                             finalized_checkpoint,
                                             finalized_checkpoint_proof,
                                             // Outputs:
                                             new_previous_justified_checkpoint,
                                             new_current_justified_checkpoint,
                                             new_finalized_checkpoint,
                                             new_justification_bits);

        std::cout << "outputs:\n";
        std::cout << "new_previous_justified_checkpoint: " << new_previous_justified_checkpoint << "\n";
        std::cout << "new_current_justified_checkpoint: " << new_current_justified_checkpoint << "\n";
        std::cout << "new_finalized_checkpoint: " << new_finalized_checkpoint << "\n";
        std::cout << "new_justification_bits: " << new_justification_bits << "\n";

    }
}

int main(int argc, char* argv[]) {

    test_circuit_sample_data();
    test_circuit_ssz_json();

    return 0;
}
