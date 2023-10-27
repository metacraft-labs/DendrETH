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

using llvm::yaml::Output;
using llvm::yaml::Input;
using llvm::yaml::MappingTraits;
using llvm::yaml::IO;

using namespace circuit_byte_utils;
using namespace file_utils;

using std::cout;

uint64_t compute_shuffled_index(
        uint64_t index,
        uint64_t index_count,
        std::array<Byte, 32> seed,
        int SHUFFLE_ROUND_COUNT = 90) {
    assert_true(index < index_count);

    std::array<Byte, 32+1+4> source_buffer{};

    //!!! sha256_to_bytes_array(seed, source_buffer);
    std::copy(seed.begin(), seed.end(), source_buffer.begin());

    // Swap or not (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
    // See the 'generalized domain' algorithm on page 3
    for(Byte current_round = 0; current_round < SHUFFLE_ROUND_COUNT; current_round++) {
        source_buffer[32] = current_round;

        //!!! auto eth2digest = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.begin() + 33);
        std::array<Byte, 32> eth2digest_bytes;
        picosha2::hash256(source_buffer.begin(), source_buffer.begin() + 33, 
                          eth2digest_bytes.begin(), eth2digest_bytes.end());
        ///!!! sha256_to_bytes_array(eth2digest, eth2digest_bytes);
        auto first8bytes = take_n_elements<Byte, eth2digest_bytes.size(), 8>(eth2digest_bytes);
        // PrintContainer(first8bytes);
        auto first8bytes_int = bytes_to_int<uint64_t>(first8bytes);
        auto pivot = first8bytes_int % index_count;
        uint64_t flip = (pivot + index_count - index) % index_count;
        auto position = std::max(index, flip);

        auto source_buffer_additional_bytes = int_to_bytes(uint32_t(position >> 8));
        for (auto i = 0; i < 4; i++) {
            source_buffer[33 + i] = source_buffer_additional_bytes[i];
        }
        ///!!! auto source = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.end());
        std::array<Byte, 32> source_to_bytes;
        picosha2::hash256(source_buffer.begin(), source_buffer.end(), 
                          source_to_bytes.begin(), source_to_bytes.end());
        ///!!! sha256_to_bytes_array(source, source_to_bytes);
        auto byte_value = source_to_bytes[(position % 256) >> 3];
        auto bit = (byte_value >> (position % 8)) % 2;

        if(bit != 0) {
            index = flip;
        }
    }

    return index;

}

struct TestInput {
    std::string seed;
    int count;
    std::vector<uint64_t> mapping;
};

namespace llvm {
    namespace yaml {
        template <>
        struct MappingTraits<TestInput> {
          static void mapping(IO &io, TestInput &info) {
            io.mapRequired("seed",         info.seed);
            io.mapOptional("count",        info.count);
            io.mapOptional("mapping",      info.mapping);
          }
        };
    }
}

int main(int argc, char* argv[]) {

    typename hashes::sha2<256>::block_type sha;

    std::array<Byte, 32> source_buffer;

    //sha = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.end());

    auto process_test_input = [] (const std::vector<path>& cases, int SHUFFLE_ROUND_COUNT) {
        for(const auto& v : cases) {
            std::cout << v.string() << ":\n";

            std::ifstream t(v.string());
            std::string yaml_content((std::istreambuf_iterator<char>(t)),
                                      std::istreambuf_iterator<char>());

            TestInput doc;

            Input yin(yaml_content);
            yin >> doc;

            if ( yin.error() ) {
                std::cerr << "Failes to process " << v.string() << "\n";
                return false;
            }

            auto seed_bytes = byte_utils::hexToBytes<32>(doc.seed);
            // std::cout << "seed=" << doc.seed << "\n";
            // std::cout << "count=" << doc.count << "\n";
            // for(const auto& v : doc.mapping) {
            //     std::cout << v << " ";
            // }
            // std::cout << "\n\n";
            // std::cout << "seed_bytes = ";
            // for(const auto& v : seed_bytes) {
            //     std::cout << (int)v << " ";
            // }
            // std::cout << "\n";
            std::vector<uint64_t> mapping_result;
            for(size_t i = 0; i < doc.mapping.size(); i++) {
                auto result = compute_shuffled_index(i, doc.mapping.size(), seed_bytes, SHUFFLE_ROUND_COUNT);
                mapping_result.push_back(result);
            }
            for(size_t i = 0; i < mapping_result.size(); i++) {
                assert_true(mapping_result[i] == doc.mapping[i]);
            }

        }
        return true;
    };

    std::vector<path> result;
    path my_path("./consensus-spec-tests");
    find_matching_files(my_path, 
        std::vector<std::string>{
            "minimal",
            "mapping.yaml"}, result);
    
    if(!process_test_input(result, 10)) {
        return 1;
    }

    result.clear();
    find_matching_files(my_path, std::vector<std::string>{
        "mainnet",
        "mapping.yaml"}, result);

    if(!process_test_input(result, 90)) {
        return 1;
    }

    return 0;
}
