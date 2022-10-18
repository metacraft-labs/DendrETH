#!/usr/bin/env bash

# Path to the root of the project
ROOT=$(git rev-parse --show-toplevel)

# Path to the contract dir
CONTRACT_DIR=${ROOT}/contracts/cosmos/light-client

# Compile Light Client implemeted in nim.
nim-wasm c --lib:${LOCAL_NIM_LIB} --nimcache:./nimbuild \
        -o:./nimbuild/light_client.wasm /${ROOT}/beacon-light-client/nim/light_client.nim \

# Compile and optimize the cosmwasm smart contract
docker run --rm -v "${CONTRACT_DIR}":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.8 .
