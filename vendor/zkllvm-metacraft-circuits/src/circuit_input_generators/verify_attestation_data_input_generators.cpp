
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
#include "utils/attestation_utils.h"

using namespace circuit_byte_utils;
using namespace byte_utils;
using namespace file_utils;
using namespace attestation_utils;

using std::cout;

constexpr size_t MAX_KEYS = 1'000'000;

int main(int argc, char* argv[]) {

    if (argc < 2) {
        std::cerr << "Needed argument for JSON output.\n";
        return -1;
    }
    std::cout << "processing /finalizer-data/merged_234400.json\n";
    path my_path("/finalizer-data/merged_234400.json");
    std::ifstream f(my_path);
    auto data = json::parse(f);

    // Generate JSON representation to pass to circuit for each attestation.
    for (const auto& json_attestation : data["attestations"]) {

        Attestation attestation = parse_attestation(json_attestation);

        std::ofstream fout(std::string(argv[1]) + ".json");
        fout << "["
             << bytes32_to_hash_type(hexToBytes<32>("d5c0418465ffab221522a6991c2d4c0041f1b8e91d01b1ea3f6b882369f689b7"))
             << ", " << serialize(attestation)    //.dump(2)
             << ", "
             << "{\"field\": 105}"
             << "]";
        fout.flush();

        std::cout << "DONE\n";

        break;    // Temporarily only process 1 attestation.
    }

    return 0;
}
