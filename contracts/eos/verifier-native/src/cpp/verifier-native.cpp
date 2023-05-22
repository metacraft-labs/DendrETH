#include <eosio/eosio.hpp>
#include <eosio/crypto.hpp>
#include <groth16/groth16.hpp>
#include <groth16/rapiduint256.hpp>

using namespace eosio;

static constexpr uint32_t ROOT_LENGTH = 32;
static constexpr uint32_t PROOF_LENGTH = 384;

static const char hex_chars[16] = {'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'};

class [[eosio::contract("verifier-native")]] verifiernative : public eosio::contract
{
public:
    verifiernative(name receiver, name code, datastream<const char *> ds)
        : contract(receiver, code, ds) {}

    // TODO: Make sure only we can instantiate
    [[eosio::action]] void instantiate(
        name key,
        const std::vector<uint8_t> &current_header_hash,
        const uint64_t &current_slot, const std::vector<uint8_t> &domain)
    {
        data_index verifier_data(get_self(), get_first_receiver().value);
        auto iterator = verifier_data.find(key.value);
        check(iterator == verifier_data.end(),
              "DendrETH verifier already instantiated");

        if (iterator == verifier_data.end())
        {
            verifier_data.emplace(key, [&](auto &row)
            {
                row.key = key;
                row.current_index = 0;
                std::vector<std::vector<uint8_t>> new_optimistic_header_roots =
                    row.new_optimistic_header_roots;
                std::vector<std::vector<uint8_t>> new_finalized_header_roots =
                    row.new_finalized_header_roots;
                std::vector<std::vector<uint8_t>> new_execution_state_roots =
                    row.new_execution_state_roots;
                std::vector<uint64_t> current_slots = row.current_slot;
                current_slots.push_back(current_slot);

                new_optimistic_header_roots.push_back(current_header_hash);
                std::vector<uint8_t> zeros = {};
                new_finalized_header_roots.push_back(zeros);
                new_execution_state_roots.push_back(zeros);
                for (int i = 1; i < 32; i++) {
                  new_optimistic_header_roots.push_back(zeros);
                  new_finalized_header_roots.push_back(zeros);
                  new_execution_state_roots.push_back(zeros);
                  current_slots.push_back(0);
                }
                row.new_optimistic_header_roots = new_optimistic_header_roots;
                row.new_finalized_header_roots = new_finalized_header_roots;
                row.new_execution_state_roots = new_execution_state_roots;
                row.current_slot = current_slots;
                row.domain = domain;
                });
        }
    }

    [[eosio::action]] void update(
        name key,
        const std::vector<std::string> proof_a,
        const std::vector<std::string> proof_b,
        const std::vector<std::string> proof_c,
        const std::vector<uint8_t> &new_optimistic_header_root,
        const std::vector<uint8_t> &new_finalized_header_root,
        const std::vector<uint8_t> &new_execution_state_root,
        const uint64_t &new_slot)
    {

        data_index verifier_data(get_self(), get_first_receiver().value);
        auto iterator = verifier_data.find(key.value);

        check(iterator != verifier_data.end(),
              "DendrETH verifier not instantiated");
        // Prepare data for the nim verifier function
        char *concateneted = new char[192];

        verifier_data.modify(iterator, key, [&](auto &row)
                             {
      std::copy(row.new_optimistic_header_roots[row.current_index].begin(),
                row.new_optimistic_header_roots[row.current_index].end(),
                concateneted);
      std::copy(new_optimistic_header_root.begin(),
                new_optimistic_header_root.end(),
                concateneted + 32);
      std::copy(new_finalized_header_root.begin(),
                new_finalized_header_root.end(),
                concateneted + 64);
      std::copy(new_execution_state_root.begin(),
                new_execution_state_root.end(),
                concateneted + 96);

      std::copy(row.domain.begin(), row.domain.end(), concateneted + 160);

      uint8_t *p = (uint8_t *)&new_slot;
      for (int i = 0; i < 8; i++) {
        concateneted[152 + i] = p[7 - i];
      }

      std::string concatenetedStr;

      for (int i = 0; i < 192; i++) {
        char const byte = concateneted[i];
        concatenetedStr += hex_chars[ ( byte & 0xF0 ) >> 4 ];
        concatenetedStr += hex_chars[ ( byte & 0x0F ) >> 0 ];
      }

      auto commitment = sha256(concateneted, 192);
      auto commitmentArr = commitment.extract_as_byte_array();

      std::string commitmentHex;

      for(int i = 0; i < 32; ++i ) {
        char const byte = commitmentArr[i];
        commitmentHex += hex_chars[ ( byte & 0xF0 ) >> 4 ];
        commitmentHex += hex_chars[ ( byte & 0x0F ) >> 0 ];
      }

      char buffer[32];
      rapid_uint256_basic::from_hex(commitmentHex, (char*)(&buffer[0]), 32);
      char firstStr[100];

      rapid_uint256_basic::u256_t first;

      rapid_uint256_basic::readu256BE((uint8_t*)(&buffer[0]), &first);

      rapid_uint256_basic::shiftr256(&first, 3, &first);

      rapid_uint256_basic::tostring256(&first, 10, (char*)(&firstStr[0]), 100);

      rapid_uint256_basic::tostring256(&first, 16, (char*)(&firstStr[0]), 100);

      std::string secondHex;

      for(int i = 0; i < 63; i++) {
        secondHex += "0";
      }

      secondHex += (char)((commitmentArr[31] & ((1 << 3) - 1)) + '0');

      std::string firstHex = std::string(firstStr);

      if(firstHex.length() < 64) {
        firstHex.insert(firstHex.begin(), 64 - firstHex.length(), '0');
      }

      bool result = groth16::verify_groth16_proof({firstHex, secondHex}, proof_a, proof_b, proof_c)==1;

      check(result, "Verification failed. Incorrect update");
      if (row.current_index == 31) {
        row.current_index = 0;
      } else {
        row.current_index++;
      }

      row.new_optimistic_header_roots[row.current_index] =
          new_optimistic_header_root;
      row.new_finalized_header_roots[row.current_index] =
          new_finalized_header_root;
      row.new_execution_state_roots[row.current_index] =
          new_execution_state_root;
      row.current_slot[row.current_index] = new_slot; });
    }

private:
    struct [[eosio::table]] verifierData
    {

        name key;
        uint32_t current_index;
        std::vector<std::vector<uint8_t>> new_optimistic_header_roots;
        std::vector<std::vector<uint8_t>> new_finalized_header_roots;
        std::vector<std::vector<uint8_t>> new_execution_state_roots;
        std::vector<uint64_t> current_slot;
        std::vector<uint8_t> domain;
        uint8_t smtElse;

        uint64_t primary_key() const { return key.value; }
    };
    using data_index = eosio::multi_index<"verifierdata"_n, verifierData>;
};
