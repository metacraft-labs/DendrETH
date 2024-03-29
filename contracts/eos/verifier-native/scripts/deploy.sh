set -o errexit -o nounset -o pipefail

# Check if the account name was provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <account_name> [rpc_endpoint]"
    exit 1
fi

DENDRETH_DIR=$(git rev-parse --show-toplevel)
DENDRETH_ACCOUNT_IN_EOS="$1"
RPC_ENDPOINT=${2:-}
CONTRACTS_DIR=${DENDRETH_DIR}/contracts/eos
VERFIER_CONTRACT_DIR=${CONTRACTS_DIR}/verifier-native
VERFIER_CONTRACT_BUILD_DIR=${VERFIER_CONTRACT_DIR}/build

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

# ------------------------------------------------------------
# Deploy the contract
# ------------------------------------------------------------

echo ""
if [ -z "$RPC_ENDPOINT" ]; then
  run_command "Deploying the contract..." \
    cleos set contract \
      ${DENDRETH_ACCOUNT_IN_EOS} \
      ${CONTRACTS_DIR} \
      ${WASM_CONTRACT} \
      ${ABI_FILE} \
      -p ${DENDRETH_ACCOUNT_IN_EOS}@active
else
  run_command "Deploying the contract..." \
    cleos --url ${RPC_ENDPOINT} set contract \
      ${DENDRETH_ACCOUNT_IN_EOS} \
      ${CONTRACTS_DIR} \
      ${WASM_CONTRACT} \
      ${ABI_FILE} \
      -p ${DENDRETH_ACCOUNT_IN_EOS}@active
fi

