#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail


DENDRETH_DIR=$(git rev-parse --show-toplevel)
DENDRETH_ACCOUNT_IN_EOS="dendreth"
CONTRACTS_DIR=${DENDRETH_DIR}/contracts/eos
VERFIER_CONTRACT_DIR=${CONTRACTS_DIR}/verifier
VERFIER_CONTRACT_BUILD_DIR=${VERFIER_CONTRACT_DIR}/build

NIM_VERIFEIR_FILE=${DENDRETH_DIR}/beacon-light-client/nim/verifier/verifier.nim
CPP_FILE=${VERFIER_CONTRACT_DIR}/src/cpp/verifier.cpp
WASM_CONTRACT=${VERFIER_CONTRACT_BUILD_DIR}/verifier.wasm
ABI_FILE=${VERFIER_CONTRACT_BUILD_DIR}/verifier.abi


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

# ------------------------------------------------------------
# Build the contract
# ------------------------------------------------------------

# Compile the nim files
run_command "Compiling the nim files" \
  nim-wasm c \
    --lib:${LOCAL_NIM_LIB} \
    --nimcache:${VERFIER_CONTRACT_BUILD_DIR} \
    -o:${VERFIER_CONTRACT_BUILD_DIR}/nim-verifier.wasm \
    ${NIM_VERIFEIR_FILE}

# Get the list of compiled to c nim files
NIM_COMPILED_OBJECT_FILES=$(jq -r '.link' ${VERFIER_CONTRACT_BUILD_DIR}/nim-verifier.json | jq -r '.[]')
for NIM_COMPILED_OBJECT_FILE in ${NIM_COMPILED_OBJECT_FILES}
do
  NIM_COMPILED_C_FILES+=$(echo ${NIM_COMPILED_OBJECT_FILE} | rev | cut -c 3- | rev)
  NIM_COMPILED_C_FILES+=$(echo " ")
done

# Compile the whole contract
run_command "Compiling the whole contract..." \
  cdt-cpp \
    -abigen \
    --stack-size 32768  \
    -o ${WASM_CONTRACT} \
    ${CPP_FILE} \
    ${NIM_COMPILED_C_FILES} \
    -I${LOCAL_NIM_LIB}

echo "Contract compiled successfully!"
echo "WASM contract: ${WASM_CONTRACT}"
echo "ABI file: ${ABI_FILE}"
