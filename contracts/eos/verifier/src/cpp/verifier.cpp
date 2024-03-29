
#include <eosio/eosio.hpp>

using namespace eosio;

static constexpr uint32_t ROOT_LENGTH = 32;
static constexpr uint32_t VERIFICATION_KEY_LENGTH = 1152;
static constexpr uint32_t PROOF_LENGTH = 384;

extern "C" {
bool makePairsAndVerify(std::array<uint8_t, VERIFICATION_KEY_LENGTH> *vk,
                        std::array<uint8_t, PROOF_LENGTH> *prf,
                        std::array<uint8_t, ROOT_LENGTH> *currentHeaderHash,
                        std::array<uint8_t, ROOT_LENGTH> *newOptimisticHeader,
                        std::array<uint8_t, ROOT_LENGTH> *newFinalizedHeader,
                        std::array<uint8_t, ROOT_LENGTH> *newExecutionStateRoot,
                        std::array<uint8_t, 8> *currentSlot,
                        std::array<uint8_t, ROOT_LENGTH> *domain);
}

class [[eosio::contract("verifier")]] verifier : public eosio::contract {
public:
  verifier(name receiver, name code, datastream<const char *> ds)
      : contract(receiver, code, ds) {}

  // TODO: Make sure only we can instantiate
  [[eosio::action]] void instantiate(
      name key,
      const std::vector<uint8_t> &verification_key,
      const std::vector<uint8_t> &current_header_hash,
      const uint64_t &current_slot, const std::vector<uint8_t> &domain) {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto iterator = verifier_data.find(key.value);
    check(iterator == verifier_data.end(),
          "DendrETH verifier already instantiated");
    if (iterator == verifier_data.end()) {
      verifier_data.emplace(key, [&](auto &row) {
        row.key = key;
        row.current_index = 0;
        row.vk = verification_key;
        std::vector<std::vector<uint8_t>> new_optimistic_header_roots =
            row.new_optimistic_header_roots;
        std::vector<std::vector<uint8_t>> new_finalized_header_roots =
            row.new_finalized_header_roots;
        std::vector<std::vector<uint8_t>> new_execution_state_roots =
            row.new_execution_state_roots;
        new_optimistic_header_roots.push_back(current_header_hash);
        std::vector<uint8_t> zeros = {};
        new_finalized_header_roots.push_back(zeros);
        new_execution_state_roots.push_back(zeros);
        for (int i = 1; i < 32; i++) {
          new_optimistic_header_roots.push_back(zeros);
          new_finalized_header_roots.push_back(zeros);
          new_execution_state_roots.push_back(zeros);
        }
        row.new_optimistic_header_roots = new_optimistic_header_roots;
        row.new_finalized_header_roots = new_finalized_header_roots;
        row.new_execution_state_roots = new_execution_state_roots;
        row.current_slot = current_slot;
        row.domain = domain;
      });
    }
  }

  [[eosio::action]] void update(
      name key,
      const std::vector<uint8_t> &proof,
      const std::vector<uint8_t> &new_optimistic_header_root,
      const std::vector<uint8_t> &new_finalized_header_root,
      const std::vector<uint8_t> &new_execution_state_root,
      const uint64_t &new_slot) {

    data_index verifier_data(get_self(), get_first_receiver().value);
    auto iterator = verifier_data.find(key.value);

    check(iterator != verifier_data.end(),
          "DendrETH verifier not instantiated");
    // Prepare data for the nim verifier function
    std::array<uint8_t, VERIFICATION_KEY_LENGTH> _vk;
    std::array<uint8_t, PROOF_LENGTH> _prf;
    std::array<uint8_t, ROOT_LENGTH> _current_header_root;
    std::array<uint8_t, ROOT_LENGTH> _new_optimistic_header_root;
    std::array<uint8_t, ROOT_LENGTH> _new_finalized_header_root;
    std::array<uint8_t, ROOT_LENGTH> _new_execution_state_root;
    std::array<uint8_t, 8> _new_slot;
    std::array<uint8_t, ROOT_LENGTH> _domain;

    verifier_data.modify(iterator, key, [&](auto &row) {
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
      std::copy(row.domain.begin(), row.domain.end(), _domain.begin());
      uint8_t *p = (uint8_t *)&new_slot;
      for (int i = 0; i < 8; i++) {
        _new_slot[i] = p[i];
      }
      // check(false,
      //       "Stoped here !!!!!!!!" );

      bool result = makePairsAndVerify(
          &_vk, &_prf, &_current_header_root, &_new_optimistic_header_root,
          &_new_finalized_header_root, &_new_execution_state_root, &_new_slot,
          &_domain);

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
      row.current_slot = new_slot;
    });
  }
  void printhelper(const std::array<uint8_t, 32> &_current_header_root) {
    eosio::print("[");
    for (int i = 0; i < 32; i++) {
      eosio::print(_current_header_root[i]);
      if (i != 31) {
        eosio::print(",");
      }
    }
    eosio::print("]");
  }

  [[eosio::action]] void printheader(name key) {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto &result = verifier_data.get(key.value);
    verifier_data.modify(result, key, [&](auto &row) {
      std::array<uint8_t, 32> _current_header_root;

      std::copy(row.new_optimistic_header_roots[row.current_index].begin(),
                row.new_optimistic_header_roots[row.current_index].end(),
                _current_header_root.begin());
      printhelper(_current_header_root);
    });
  }

  [[eosio::action]] void printheaders(name key) {
    data_index verifier_data(get_self(), get_first_receiver().value);
    auto &result = verifier_data.get(key.value);
    verifier_data.modify(result, key, [&](auto &row) {
      std::array<uint8_t, 32> _current_header_root;

      for (int pos = 0; pos < 32; pos++) {
        std::copy(row.new_optimistic_header_roots[pos].begin(),
                  row.new_optimistic_header_roots[pos].end(),
                  _current_header_root.begin());
        printhelper(_current_header_root);
      }
    });
  }

private:
  struct [[eosio::table]] verifierData {

    name key;
    std::vector<uint8_t> vk;
    uint32_t current_index;
    std::vector<std::vector<uint8_t>> new_optimistic_header_roots;
    std::vector<std::vector<uint8_t>> new_finalized_header_roots;
    std::vector<std::vector<uint8_t>> new_execution_state_roots;
    uint8_t current_slot;
    std::vector<uint8_t> domain;
    uint8_t smtElse;

    uint64_t primary_key() const { return key.value; }
  };
  using data_index = eosio::multi_index<"verifierdata"_n, verifierData>;
};
