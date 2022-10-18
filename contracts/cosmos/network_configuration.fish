
set CHAIN_ID "malaga-420"
set TESTNET_NAME "malaga-420"
set FEE_DENOM "umlg"
set STAKE_DENOM "uand"
set BECH32_HRP "wasm"
set WASMD_VERSION "v0.27.0"
set CONFIG_DIR ".wasmd"
set BINARY "wasmd"

set COSMJS_VERSION "v0.28.1"
set GENESIS_URL "https://raw.githubusercontent.com/CosmWasm/testnets/master/malaga-420/config/genesis.json"

set RPC "https://rpc.malaga-420.cosmwasm.com:443"
set API "https://api.malaga-420.cosmwasm.com"
set FAUCET "https://faucet.malaga-420.cosmwasm.com"

set NODE --node $RPC

set TXFLAG $NODE --chain-id $CHAIN_ID --gas-prices 0.25umlg --gas auto --gas-adjustment 1.3

