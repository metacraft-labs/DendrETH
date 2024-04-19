#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail


DENDRETH_DIR=$(git rev-parse --show-toplevel)
DENDRETH_ACCOUNT_IN_EOS="dendreth"
CONTRACTS_DIR=${DENDRETH_DIR}/contracts/eos
VERFIER_CONTRACT_DIR=${CONTRACTS_DIR}/verifier-native
VERFIER_CONTRACT_BUILD_DIR=${VERFIER_CONTRACT_DIR}/build

CPP_FILE=${VERFIER_CONTRACT_DIR}/src/cpp/verifier-native.cpp
INCLUDE_FOLDER=${VERFIER_CONTRACT_DIR}/src/cpp/include
WASM_CONTRACT=${VERFIER_CONTRACT_BUILD_DIR}/verifier-native.wasm
ABI_FILE=${VERFIER_CONTRACT_BUILD_DIR}/verifier-native.abi


function run_command {
  echo -e "┌───  \033[1mstart \033[34m$1\033[0m ────╌╌╌"
  {
    shift
    echo ╰─➤
    echo $* | sed 's/^/  /'
    echo ""
    eval "$@"
    # TODO: If the command fails, pass the exit code to the caller
  } 2>&1 | fmt -s -w 80 | sed 's/^/│  /'
  echo -e "└────╼ \033[1mend \033[34m$1\033[0m ────╌╌╌"
  echo ""
}

mkdir -p ${VERFIER_CONTRACT_BUILD_DIR}

# Compile the whole contract
run_command "Compiling the whole contract..." \
  cdt-cpp \
    -abigen \
    --stack-size 32768  \
    -o ${WASM_CONTRACT} \
    -I=${INCLUDE_FOLDER} \
    ${CPP_FILE}

echo "Contract compiled successfully!"
echo "WASM contract: ${WASM_CONTRACT}"
echo "ABI file: ${ABI_FILE}"
