#pragma once

#include <stdlib.h>
#include <array>
#include "circuit_utils/base_types.h"
#include "circuit_utils/static_vector.h"
#include <sstream>
#include <string>
#include <string_view>
#include <sstream>
#include <iomanip>
#include <cctype>

namespace byte_utils {

    Byte hexToByte(const char* hex) {

        Byte first_nibble = 0;
        Byte second_nibble = 0;

        auto convert = [](char hex, Byte& nibble) {
            if (hex >= '0' && hex <= '9') {
                nibble = hex - '0';
            } else if (hex >= 'a' && hex <= 'f') {
                nibble = hex - 'a' + 10;
            } else {
                assert_true(false && "Not a valid hex character");
            }
        };

        convert(std::tolower(hex[0]), first_nibble);
        convert(std::tolower(hex[1]), second_nibble);

        return (first_nibble << 4) + second_nibble;
    }

#define PrintContainer(val)              \
    do {                                 \
        std::cout << #val << ": ";       \
        byte_utils::printContainer(val); \
    } while (0)
    template<typename T>
    void printContainer(const T& a) {
        for (const auto& v : a) {
            std::cout << (int)v << " ";
        }
        std::cout << "\n";
    }

    template<size_t SIZE>
    std::string bytesToHex(static_vector<Byte, SIZE> uint8a) {
        std::string s = "";
        std::ostringstream oss;
        oss << std::setfill('0');

        for (int i = 0; i < SIZE; ++i) {
            oss << std::setw(2) << std::hex << static_cast<int>(uint8a[i]);
        }

        return oss.str();
    }

    void formatHex(std::string_view& str) {
        static const std::string_view prefix = "0x";
        if (str.compare(0, prefix.size(), prefix) == 0) {
            str.remove_prefix(prefix.size());
        }
    }

    template<long unsigned int SIZE>
    static_vector<Byte, SIZE> hexToBytes(const std::string& hex_str) {
        std::string_view hex(hex_str);
        formatHex(hex);
        assert_true(hex.length() == (2 * SIZE));

        static_vector<Byte, SIZE> bytes;
        for (size_t i = 0; i < bytes.size(); ++i) {
            auto hexByte = hex.substr(i * 2, 2);
            bytes[i] = hexToByte(hexByte.data());
        }
        return bytes;
    }

    uint64_t stringToUint64(std::string val) {
        uint64_t retval = 0;
        retval = strtoll(val.c_str(), nullptr, 10);
        return retval;
    }

    JustificationBitsVariable hexToBitsVariable(std::string hex) {
        JustificationBitsVariable retval {};
        auto bits = hexToBytes<1>(hex);
        for (int64_t i = 0; i < (int64_t)retval.bits.size(); ++i) {
            retval.bits[i] = (bits[0] % 2);
            bits[0] /= 2;
        }
        return retval;
    }

}    // namespace byte_utils
