#!/usr/bin/env bash
set -euxo pipefail

#wasmd keys add wallet

#JSON=$(jq -n --arg addr $(wasmd keys show -a wallet) '{"denom":"umlg","address":$addr}') && curl -X POST --header "Content-Type: application/json" --data "$JSON" https://faucet.malaga-420.cosmwasm.com/credit

CHAIN_ID="malaga-420"
RPC="https://rpc.malaga-420.cosmwasm.com:443"

NODE="--node ${RPC}"
TXFLAG="${NODE} --chain-id ${CHAIN_ID} --gas-prices 0.25umlg --gas auto --gas-adjustment 1.3"


# Path to the root of the project
ROOT=$(git rev-parse --show-toplevel)

# Path to the contract dir
CONTRACT_DIR=${ROOT}/verifier

# Compile Light Client implemeted in nim.
nim-wasm c --lib:${LOCAL_NIM_LIB} --nimcache:./nimcache --d:lightClientCosmos \
        -o:./nimcache/verifier.wasm ${ROOT}/libs/nim/nim-groth16-verifier/verify.nim \

# Compile and optimize the cosmwasm smart contract
docker run -t --rm -v "${CONTRACT_DIR}":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.8 .

RES=$(wasmd tx wasm store ${CONTRACT_DIR}/artifacts/verifier.wasm --from wallet $TXFLAG -y --output json -b block)
echo "$RES" >> contracts_stored.log

CODE_ID=$(echo "$RES" | jq -r '.logs[0].events[-1].attributes[0].value')

# BOOTSTRAP_DATA=""
INIT='{"vkey":15, "currentHeader":3}'

wasmd tx wasm instantiate $CODE_ID "$INIT" --from wallet --label "name service" ${TXFLAG} -y --no-admin
sleep 8

CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')

NAME_QUERY="{\"store\": {}}"
wasmd query wasm contract-state smart $CONTRACT "$NAME_QUERY" $NODE --output json

UPDATE_DATA=""
UPDATE='{"update":{"proofInput":3,"newHeader":4}}'
wasmd tx wasm execute $CONTRACT "$UPDATE" --amount 999umlg --from wallet $TXFLAG -y
sleep 8

NAME_QUERY="{\"store\": {}}"
wasmd query wasm contract-state smart $CONTRACT "$NAME_QUERY" $NODE --output json

NAME_QUERY="{\"header\": {}}"
wasmd query wasm contract-state smart $CONTRACT "$NAME_QUERY" $NODE --output json
