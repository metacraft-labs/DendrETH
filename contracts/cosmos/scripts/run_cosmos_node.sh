#!/usr/bin/env bash

DENDRETH_DIR=$(git rev-parse --show-toplevel)
COSMOS_CONTRACTS_DIR=${DENDRETH_DIR}/contracts/cosmos
COSMOS_DATA_DIR=${COSMOS_CONTRACTS_DIR}/.node-data-dir/
SETUP_WASMD_SCRIPT=${COSMOS_CONTRACTS_DIR}/scripts/setup_wasmd.sh
START_COSMOS_NODE_SCIRPTS=${COSMOS_CONTRACTS_DIR}/scripts/start_node.sh
ACC_SETUP_SCRIPT=${COSMOS_CONTRACTS_DIR}/scripts/add_account.sh

CMD=$1; shift

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

start_node() {
  bash ${START_COSMOS_NODE_SCIRPTS} > ${COSMOS_DATA_DIR}/cosmos_node.log 2>&1 &
}

stop_node() {
	ps aux |grep wasmd |
		awk '{print $2}' |
		xargs -I % sh -c '[ ! -z % ] && kill %'
}


start_cosmos_node() {
  run_command "Preparing 'wasmd'" \
    bash ${SETUP_WASMD_SCRIPT}

  run_command "Starting Cosmos node" \
    start_node

  sleep 15 #  Make sure the node has started

  run_command "Creating and funding account" \
    bash ${ACC_SETUP_SCRIPT}

  sleep 10 # Make sure the new account is funded
}

[ "$CMD" == "start" ] && {
	start_cosmos_node
	exit 0
}

[ "$CMD" == "stop" ] && {
	stop_node
	exit 0
}
