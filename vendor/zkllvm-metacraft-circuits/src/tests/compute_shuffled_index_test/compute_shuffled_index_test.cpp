#include "circuit_utils/circuit_byte_utils.h"
#include "circuits_impl/compute_shuffled_index_impl.h"

#include <algorithm>
#include <array>

#include <iostream>
#include <fstream>
#include <streambuf>

#include "utils/picosha2.h"
#include "utils/byte_utils.h"
#include "utils/file_utils.h"

#include "yaml-cpp/yaml.h"

using namespace circuit_byte_utils;
using namespace file_utils;

int main(int argc, char* argv[]) {

    auto process_test_input = [](const std::vector<path>& cases, int SHUFFLE_ROUND_COUNT) {
        for (const auto& v : cases) {
            std::cout << v.string() << ":\n";

            YAML::Node config = YAML::LoadFile(v.string());

            auto seed_bytes = byte_utils::hexToBytes<32>(config["seed"].as<std::string>());

            std::vector<uint64_t> mapping_result;
            for (size_t i = 0; i < config["mapping"].size(); i++) {
                auto result = compute_shuffled_index_impl(i, config["mapping"].size(), seed_bytes, SHUFFLE_ROUND_COUNT);
                mapping_result.push_back(result);
            }
            for (size_t i = 0; i < mapping_result.size(); i++) {
                assert_true(mapping_result[i] == config["mapping"][i].as<int>());
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
