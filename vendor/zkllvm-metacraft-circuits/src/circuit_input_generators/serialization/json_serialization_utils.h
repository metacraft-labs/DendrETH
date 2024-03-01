#pragma once

#include <sstream>

#include "json/json.hpp"
using namespace nlohmann;

nlohmann::json pack_int_json(uint64_t val) {
    nlohmann::json j;
    j["int"] = val;
    return j;
}

std::ostream& operator<<(std::ostream& dest, __uint128_t value) {
    std::ostream::sentry s(dest);
    if (s) {
        char buffer[128];
        char* d = std::end(buffer);
        do {
            --d;
            *d = "0123456789"[value % 10];
            value /= 10;
        } while (value != 0);
        int len = std::end(buffer) - d;
        if (dest.rdbuf()->sputn(d, len) != len) {
            dest.setstate(std::ios_base::badbit);
        }
    }
    return dest;
}

// Example expected output:
// {"vector": [{"field": "326522724692461750427768532537390503835"},{"field":
// "89059515727727869117346995944635890507"}]},
nlohmann::json bytes32_to_hash_type(const Bytes32& bytes) {
    union Convert {
        __uint128_t v[2];
        Byte bytes[32];
    } c;
    for (size_t i = 0; i < bytes.size(); i++) {
        c.bytes[bytes.size() - i - 1] = bytes[i];
    }
    std::stringstream val[2];
    nlohmann::json field[2];
    nlohmann::json result;

    for (size_t i = 0; i < 2; i++) {
        val[i] << c.v[i];
        field[i]["field"] = val[i].str();
        result["vector"].push_back(field[i]);
    }

    return result;
}

template<size_t N>
static_vector<Byte, N> json_to_byte_array(const nlohmann::json& j) {
    static_vector<Byte, N> result;
    size_t i = 0;
    for (const auto& v : j["array"]) {
        result[i++] = v["int"];
    }
    return result;
}

template<typename T>
nlohmann::json serialize(const T& val);

template<size_t N, bool F>
nlohmann::json serialize_vector(const static_vector<Byte, N, F>& bytes) {
    nlohmann::json res_array;
    for (size_t i = 0; i < N; i++) {
        res_array["array"].push_back(pack_int_json(bytes[i]));
    }
    nlohmann::json result;
    result["struct"].push_back(res_array);
    return result;
}

nlohmann::json serialize_vector(HashType h) {
    return bytes32_to_hash_type(h);
}

template<typename C, size_t S, bool B>
nlohmann::json serialize_vector(const static_vector<C, S, B>& v) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json((size_t)v.size()));
    nlohmann::json elements;
    for (size_t i = 0; i < S; i++) {
        elements["array"].push_back(serialize(v[i]));
    }
    result["struct"].push_back(elements);
    return result;
}

template<typename C1, size_t S1, bool B1, size_t S, bool B>
nlohmann::json serialize_vector(const static_vector<static_vector<C1, S1, B1>, S, B>& v) {
    nlohmann::json result;
    result["struct"].push_back(pack_int_json((size_t)v.size()));
    nlohmann::json elements;
    for (size_t i = 0; i < S; i++) {
        elements["array"].push_back(serialize_vector(v[i]));
    }
    result["struct"].push_back(elements);
    return result;
}

template<size_t N>
nlohmann::json byte_array_to_json(const static_vector<Byte, N>& bytes) {
    nlohmann::json res_array;
    for (size_t i = 0; i < N; i++) {
        res_array["array"].push_back(pack_int_json(bytes[i]));
    }
    nlohmann::json result;
    result["struct"].push_back(res_array);
    return result;
}
