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
# Deploy the contract
# ------------------------------------------------------------

echo ""
run_command "Deploying the contract..." \
  cleos set contract \
    ${DENDRETH_ACCOUNT_IN_EOS} \
    ${CONTRACTS_DIR} \
    ${WASM_CONTRACT} \
    ${ABI_FILE} \
    -p ${DENDRETH_ACCOUNT_IN_EOS}@active

