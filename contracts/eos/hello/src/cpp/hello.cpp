#include <eosio/eosio.hpp>

extern "C" {
    char* helloFromNim();
}

class [[eosio::contract]] hello : public eosio::contract {

    public:
        using eosio::contract::contract;
        [[eosio::action]] void hi( eosio::name user ) {
            print(helloFromNim(), ", ", user);
        }
};
