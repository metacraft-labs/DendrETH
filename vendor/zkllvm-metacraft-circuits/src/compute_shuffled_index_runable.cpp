#include <nil/crypto3/hash/algorithm/hash.hpp>
#include <nil/crypto3/hash/sha2.hpp>

#include <algorithm>
#include <array>

#include "byte_utils.h"

#include "boost/filesystem.hpp"   // includes all needed Boost.Filesystem declarations
#include <iostream>               // for std::cout
using namespace boost::filesystem;// for ease of tutorial presentation;
                                  //  a namespace alias is preferred practice in real code

using namespace byte_utils;

#include <llvm/ObjectYAML/YAML.h>

using std::cout;

using llvm::yaml::MappingTraits;
using llvm::yaml::IO;

constexpr unsigned char SHUFFLE_ROUND_COUNT = 10;

uint64_t compute_shuffled_index(
        uint64_t index,
        uint64_t index_count,
        sha256_t seed) {
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

    using llvm::yaml::Output;
/*
seed: '0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88'
count: 5
mapping: [4, 1, 0, 3, 2]
*/
    TestInput tom;
    tom.seed = "0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88";
    tom.count = 5;
    tom.mapping = {4, 1, 0, 3, 2};

    Output yout(llvm::outs());
    yout << tom;

    ///////////////////////////////////////////////////////////////////////////////////////////////////

    using llvm::yaml::Input;

    TestInput doc;

    auto my_yaml = std::string(
R"(seed: '0x2c7c329908222b0e98b0dc09c8e92c6f28b2abb4c6b5300f4244e6b740311f88'
count: 10
mapping: [9, 5, 3, 8, 4, 6, 2, 0, 7, 1])");

    Input yin(my_yaml);
    yin >> doc;

    cout << "\nmy_yaml = " << my_yaml << "\n";

    if ( yin.error() )
      return 1;

    // Process read document
    cout << "doc.seed = " << doc.seed << "\n";
    cout << "doc.count = " << doc.count << "\n";
    cout << "doc.mapping = [";
    for(auto it = doc.mapping.begin(); it != doc.mapping.end(); it++) {
        cout << *it << ", ";
    }
    cout << "]\n";

    std::vector<path> result;
    path my_path("/tmp");
    find_matching_files(my_path, std::vector<std::string>{"2", "mapping.yaml"}, result);
    for(const auto& v : result) {
        std::string s = v.string();
        std::cout << s << "\n\n";
    }

    return 0;
}
