
#include <eosio/eosio.hpp>

using namespace eosio;

static constexpr uint32_t ROOT_LENGTH = 32;
static constexpr uint32_t VERIFICATION_KEY_LENGTH = 1152;
static constexpr uint32_t PROOF_LENGTH = 384;

extern "C" {
bool makePairsAndVerify(std::array<uint8_t, 1152> *vk,
                        std::array<uint8_t, 384> *prf,
                        std::array<uint8_t, 32> *currentHeaderHash,
                        std::array<uint8_t, 32> *newOptimisticHeader,
                        std::array<uint8_t, 32> *newFinalizedHeader,
                        std::array<uint8_t, 32> *newExecutionStateRoot);
}
class [[eosio::contract("verifier")]] verifier : public eosio::contract {
public:
  verifier(name receiver, name code, datastream<const char *> ds)
      : contract(receiver, code, ds) {}

  // TODO: Make sure only we can instantiate
  [[eosio::action]] void instantiate(std::vector<uint8_t> verification_key,
                                     std::vector<uint8_t> current_header_hash) {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto iterator = verifier_data.find(verifier_name.value);
    check(iterator == verifier_data.end(),
          "DendrETH verifier already instantiated");
    if (iterator == verifier_data.end()) {
      verifier_data.emplace(verifier_name, [&](auto &row) {
        row.key = verifier_name;
        row.current_index = 0;
        row.vk = verification_key;
        std::vector<std::vector<uint8_t>> new_optimistic_header_roots =
            row.new_optimistic_header_roots;
        std::vector<std::vector<uint8_t>> new_finalized_header_roots =
            row.new_finalized_header_roots;
        std::vector<std::vector<uint8_t>> new_execution_state_roots =
            row.new_execution_state_roots;
        new_optimistic_header_roots.push_back(current_header_hash);
        for (int i = 1; i < 32; i++) {
          std::vector<uint8_t> zeros = {};
          new_optimistic_header_roots.push_back(zeros);
          new_finalized_header_roots.push_back(zeros);
          new_execution_state_roots.push_back(zeros);
        }
        row.new_optimistic_header_roots = new_optimistic_header_roots;
        row.new_finalized_header_roots = new_finalized_header_roots;
        row.new_execution_state_roots = new_execution_state_roots;
      });
    }
  }

  [[eosio::action]] void update(std::vector<uint8_t> proof,
                                std::vector<uint8_t> new_optimistic_header_root,
                                std::vector<uint8_t> new_finalized_header_root,
                                std::vector<uint8_t> new_execution_state_root) {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto iterator = verifier_data.find(verifier_name.value);

    check(iterator != verifier_data.end(),
          "DendrETH verifier not instantiated");
    // Prepare data for the nim verifier function
    std::array<uint8_t, 1152> _vk;
    std::array<uint8_t, 384> _prf;
    std::array<uint8_t, 32> _current_header_root;
    std::array<uint8_t, 32> _new_optimistic_header_root;
    std::array<uint8_t, 32> _new_finalized_header_root;
    std::array<uint8_t, 32> _new_execution_state_root;

    verifier_data.modify(iterator, verifier_name, [&](auto &row) {
      std::copy(row.vk.begin(), row.vk.end(), _vk.begin());
      std::copy(proof.begin(), proof.end(), _prf.begin());
      std::copy(row.new_optimistic_header_roots[row.current_index].begin(),
                row.new_optimistic_header_roots[row.current_index].end(),
                _current_header_root.begin());
      std::copy(new_optimistic_header_root.begin(),
                new_optimistic_header_root.end(),
                _new_optimistic_header_root.begin());
      std::copy(new_finalized_header_root.begin(),
                new_finalized_header_root.end(),
                _new_finalized_header_root.begin());
      std::copy(new_execution_state_root.begin(),
                new_execution_state_root.end(),
                _new_execution_state_root.begin());

      bool result = makePairsAndVerify(
          &_vk, &_prf, &_current_header_root, &_new_optimistic_header_root,
          &_new_finalized_header_root, &_new_execution_state_root);

      check(result, "Verification failed. Incorrect update");
      if (row.current_index == 31) {
        row.current_index = 0;
      } else {
        row.current_index++;
      }

      std::vector<uint8_t> _new_current_header_root(
          _current_header_root.begin(), _current_header_root.end());

      row.new_optimistic_header_roots[row.current_index] =
          _new_current_header_root;
      row.new_finalized_header_roots[row.current_index] =
          new_finalized_header_root;
      row.new_execution_state_roots[row.current_index] =
          new_execution_state_root;
    });
  }

  [[eosio::action]] void printheader() {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto &result = verifier_data.get(verifier_name.value);
    verifier_data.modify(result, verifier_name, [&](auto &row) {
      std::array<uint8_t, 32> _current_header_root;

      std::copy(row.new_optimistic_header_roots[row.current_index].begin(),
                row.new_optimistic_header_roots[row.current_index].end(),
                _current_header_root.begin());
      eosio::print("[");
      for (int i = 0; i < 32; i++) {
        eosio::print(_current_header_root[i]);
        if (i != 31) {
          eosio::print(",");
        }
      }
      eosio::print("]");
    });
  }

  [[eosio::action]] void printheaders() {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto &result = verifier_data.get(verifier_name.value);
    verifier_data.modify(result, verifier_name, [&](auto &row) {
      std::array<uint8_t, 32> _current_header_root;

      for (int pos = 0; pos < 32; pos++) {
        std::copy(row.new_optimistic_header_roots[pos].begin(),
                  row.new_optimistic_header_roots[pos].end(),
                  _current_header_root.begin());
        eosio::print("[");
        for (int i = 0; i < 32; i++) {
          eosio::print(_current_header_root[i]);
          if (i != 31) {
            eosio::print(",");
          }
        }
        eosio::print("]");
      }
    });
  }

private:
  // TODO: think of better name for this
  const name verifier_name = "dendreth"_n;

  struct [[eosio::table]] verifierData {

    name key;
    std::vector<uint8_t> vk;
    uint32_t current_index;
    std::vector<std::vector<uint8_t>> new_optimistic_header_roots;
    std::vector<std::vector<uint8_t>> new_finalized_header_roots;
    std::vector<std::vector<uint8_t>> new_execution_state_roots;
    uint8_t smtElse;

    uint64_t primary_key() const { return key.value; }
  };
  using data_index = eosio::multi_index<"verifierdata"_n, verifierData>;
};
