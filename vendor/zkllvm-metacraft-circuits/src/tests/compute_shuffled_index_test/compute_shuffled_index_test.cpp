#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "circuit_utils/circuit_byte_utils.h"
#include "circuits_impl/compute_shuffled_index_impl.h"

#include <algorithm>
#include <array>

#include <llvm/ObjectYAML/YAML.h>
#include <iostream>
#include <fstream>
#include <streambuf>

#include "utils/picosha2.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"

#include "llvm/Support/JSON.h"

#include "circuits_impl/verify_attestation_data_imp.h"

using llvm::yaml::Input;
using llvm::yaml::IO;
using llvm::yaml::MappingTraits;
using llvm::yaml::Output;

using namespace circuit_byte_utils;
using namespace file_utils;

struct TestInput {
    std::string seed;
    int count;
    std::vector<uint64_t> mapping;
};

namespace llvm {
    namespace yaml {
        template<>
        struct MappingTraits<TestInput> {
            static void mapping(IO& io, TestInput& info) {
                io.mapRequired("seed", info.seed);
                io.mapOptional("count", info.count);
                io.mapOptional("mapping", info.mapping);
            }
        };
    }    // namespace yaml
}    // namespace llvm

int main(int argc, char* argv[]) {

    auto process_test_input = [](const std::vector<path>& cases, int SHUFFLE_ROUND_COUNT) {
        for (const auto& v : cases) {
            std::cout << v.string() << ":\n";

            std::ifstream t(v.string());
            std::string yaml_content((std::istreambuf_iterator<char>(t)), std::istreambuf_iterator<char>());

            TestInput doc;

            Input yin(yaml_content);
            yin >> doc;

            if (yin.error()) {
                std::cerr << "Failed to process " << v.string() << "\n";
                return false;
            }

            auto seed_bytes = byte_utils::hexToBytes<32>(doc.seed);

            std::vector<uint64_t> mapping_result;
            for (size_t i = 0; i < doc.mapping.size(); i++) {
                auto result = compute_shuffled_index_impl(i, doc.mapping.size(), seed_bytes, SHUFFLE_ROUND_COUNT);
                mapping_result.push_back(result);
            }
            for (size_t i = 0; i < mapping_result.size(); i++) {
                assert_true(mapping_result[i] == doc.mapping[i]);
            }
        }
        return true;
    };

    std::vector<path> result;
    path my_path("/consensus-spec-tests");
    try {
        find_matching_files(my_path, std::vector<std::string> {"minimal", "shuffling", "mapping.yaml"}, result);
    } catch (const NonExistentPath& e) {
        std::cerr << "ERROR: non existing path " << e.what() << "\n";
        return 1;
    }

    if (!process_test_input(result, 10)) {
        return 1;
    }

    result.clear();

    try {
        find_matching_files(my_path, std::vector<std::string> {"mainnet", "shuffling", "mapping.yaml"}, result);
    } catch (const NonExistentPath& e) {
        std::cerr << "ERROR: non existing path " << e.what() << "\n";
        return 1;
    }

    if (!process_test_input(result, 90)) {
        return 1;
    }

    return 0;
}
