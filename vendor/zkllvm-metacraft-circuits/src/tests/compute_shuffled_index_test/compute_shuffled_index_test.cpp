#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>
#include "byte_utils.h"

#include <algorithm>
#include <array>

#include "boost/filesystem.hpp"
#include <llvm/ObjectYAML/YAML.h>
#include <iostream> 
#include <fstream>
#include <streambuf>

using namespace boost::filesystem;

using llvm::yaml::Output;
using llvm::yaml::Input;
using llvm::yaml::MappingTraits;
using llvm::yaml::IO;

using namespace byte_utils;

using std::cout;

uint64_t compute_shuffled_index(
        uint64_t index,
        uint64_t index_count,
        sha256_t seed,
        int SHUFFLE_ROUND_COUNT = 90) {
    assert_true(index < index_count);

    std::array<unsigned char, 32+1+4> source_buffer;
    uint64_t cur_idx_permuted = index;

    sha256_to_bytes_array(seed, source_buffer);

    // Swap or not (https://link.springer.com/content/pdf/10.1007%2F978-3-642-32009-5_1.pdf)
    // See the 'generalized domain' algorithm on page 3
    for(unsigned char current_round = 0; current_round < SHUFFLE_ROUND_COUNT; current_round++) {
        source_buffer[32] = current_round;

        auto eth2digest = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.begin() + 33);
        std::array<unsigned char, 32> eth2digest_bytes;
        sha256_to_bytes_array(eth2digest, eth2digest_bytes);
        auto first8bytes = take_n_elements<unsigned char, eth2digest_bytes.size(), 8>(eth2digest_bytes);
        auto first8bytes_int = bytes_to_int<uint64_t>(first8bytes);
        auto pivot = first8bytes_int % index_count;
        auto flip = ((index_count + pivot) - cur_idx_permuted) % index_count;
        auto position = std::max(cur_idx_permuted, flip);

        auto source_buffer_additional_bytes = int_to_bytes(uint32_t(position >> 8));
        for (auto i = 0; i <= 4; i++) {
            source_buffer[33 + i] = source_buffer_additional_bytes[i];
        }

        auto source = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.end());
        std::array<unsigned char, 32> source_to_bytes;
        sha256_to_bytes_array(source, source_to_bytes);
        auto byte_value = source_to_bytes[(position % 256) >> 3];
        auto bit = (byte_value >> (position % 8)) % 2;

        if(bit != 0) {
            cur_idx_permuted = flip;
        }
    }

    return cur_idx_permuted;

}

struct TestInput {
    std::string seed;
    int count;
    std::vector<int> mapping;
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

void find_matching_files( const path & dir_path,           // in this directory,
                const std::vector<std::string> & patterns, // search for this name,
                std::vector<path> & path_found )           // placing path here if found
{
    auto check_matching = [](const std::string& file_path,
                             const std::vector<std::string> & patterns) {
        for(const auto& v : patterns) {
            if(file_path.find(v) == std::string::npos) {
                return false;
            }
        }
        return true;
    };
    if ( !exists( dir_path ) ) return;
    directory_iterator end_itr; // default construction yields past-the-end
    for ( directory_iterator itr( dir_path );
            itr != end_itr;
            ++itr )
    {
        if (is_directory(itr->status()))
        {
            find_matching_files(itr->path(), patterns, path_found);
        }
        else if (check_matching(itr->path().string(), patterns))
        {
            path_found.push_back(itr->path());
        }
    }
}

int main(int argc, char* argv[]) {

    typename hashes::sha2<256>::block_type sha;

    std::array<unsigned char, 32> source_buffer;

    //sha = hash<hashes::sha2<256>>(source_buffer.begin(), source_buffer.end());

    auto process_test_input = [] (const std::vector<path>& cases) {
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

            std::cout << "seed=" << doc.seed << "\n";
            std::cout << "count=" << doc.count << "\n";
            for(const auto& v : doc.mapping) {
                std::cout << v << " ";
            }
            std::cout << "\n\n";
        }
        return true;
    };

    std::vector<path> result;
    path my_path("./consensus-spec-tests");
    find_matching_files(my_path, std::vector<std::string>{"minimal", "mapping.yaml"}, result);
    find_matching_files(my_path, std::vector<std::string>{"mainnet", "mapping.yaml"}, result);

    if(!process_test_input(result)) {
        return 1;
    }

    return 0;
}
