#!/usr/bin/env bash

DENDRETH_DIR=$(git rev-parse --show-toplevel)
DENDRETH_ACCOUNT_IN_EOS="dendreth"

EOS_DEVELOPEMENT_KEY=5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3
EOS_CONTRACT_DIR=${DENDRETH_DIR}/contracts/eos
EOS_NODE_DATA_DIR_DIR=${EOS_CONTRACT_DIR}/.node-data-dir
EOS_WALLET_DATA=${EOS_NODE_DATA_DIR_DIR}/.wallet
EOS_KEYS_DATA=${EOS_NODE_DATA_DIR_DIR}/.keys
EOS_LOGS_DIR=${EOS_NODE_DATA_DIR_DIR}/logs
EOS_GENESIS=${EOS_NODE_DATA_DIR_DIR}/genesis.json

WALLET_NAME="DendrETH-wallet"

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

isRunning() {
    s=$(echo $1 |sed 's/./[&]/') # this trick prevents grep from finding itself
    ps aux |grep --silent $s
    [ $? -eq 0 ] && echo Y || echo N
}

walletExists() {
	cleos wallet open -n ${WALLET_NAME} > /dev/null 2>&1
	[ $? -eq 0 ] && echo Y || echo N
}

walletUnlock() {
 	[[ $(cleos wallet list) != *"*"* ]] && {
 		echo "* Unlocking wallet..."
 		cat ${EOS_WALLET_DATA} | cleos wallet unlock -n ${WALLET_NAME}
 	}
 }

acct() {
	PK=$(tail -1 ${EOS_KEYS_DATA} |sed 's/Public key: //')
	for acct in "$@"; do
		cleos create account eosio $acct $PK $PK
	done
}

cleanup() {
  rm ${HOME}/eosio-wallet/./DendrETH-wallet.wallet > /dev/null 2>&1
  [ -d ${EOS_NODE_DATA_DIR_DIR} ] && find ${EOS_NODE_DATA_DIR_DIR} ! -name 'genesis.json'  -type f -exec rm -f {} +
  mkdir -p ${EOS_NODE_DATA_DIR_DIR}
  touch ${EOS_NODE_DATA_DIR_DIR}/.gitkeep
  mkdir -p ${EOS_LOGS_DIR}
  touch ${EOS_LOGS_DIR}/.gitkeep
}

quit() {
	s=$(echo $1 |sed 's/./[&]/') # this trick prevents grep from finding itself
	ps aux |grep $s |
		awk '{print $2}' |
		xargs -I % sh -c '[ ! -z % ] && kill %'
}

start_keosd() {
	nohup keosd > ${EOS_LOGS_DIR}/keosd.log 2>&1 &
}

start_nodeos() {
  nodeos \
    -e -p eosio \
    --data-dir ${EOS_NODE_DATA_DIR_DIR} \
    --config-dir ${EOS_NODE_DATA_DIR_DIR} \
    --genesis-json ${EOS_GENESIS} \
    --plugin eosio::producer_plugin \
    --plugin eosio::chain_plugin \
    --plugin eosio::http_plugin \
    --plugin eosio::state_history_plugin \
    --plugin eosio::chain_api_plugin \
    --max-transaction-time 3000000 \
    --read-only-read-window-time-us 3500000000 \
    --disable-subjective-billing=true \
    --contracts-console \
    --disable-replay-opts \
    --access-control-allow-origin='*' \
    --http-validate-host=false \
    --verbose-http-errors \
    --state-history-dir ${EOS_NODE_DATA_DIR_DIR}/shpdata \
    --trace-history \
    --chain-state-history \
    --replay-blockchain > ${EOS_LOGS_DIR}/nodeos.log 2>&1 &
}

# ------------------------------------------------------------------------------
[ "$CMD" == "stop" ] && {
  echo -e "──────  \033[1m\033[34mStopping nodeos\033[0m ──────"
	quit "nodeos"
  echo -e "──────  \033[1m\033[34mStopping keosd\033[0m ──────"
	quit "keosd"
  echo -e "──────  \033[1m\033[34mCleaning up\033[0m ──────"
  cleanup
	exit 0
}

[ "$CMD" == "start" ] && {
	start_$1
	exit 0
}

[ $(isRunning keosd) == "N" ] && {
  echo -e "──────  \033[1m\033[34mStarting keosd\033[0m ──────"
	start_keosd
}
[ $(isRunning nodeos) == "N" ] && {
  echo -e "──────  \033[1m\033[34mStarting nodeos\033[0m ──────"
	start_nodeos
	sleep 1
}

# Create the wallet if it doesn't exist
[ $(walletExists) == "N" ] && {
    run_command "Creating wallet" \
      cleos wallet create -n ${WALLET_NAME} --file ${EOS_WALLET_DATA}
}

# Unlock the wallet
walletUnlock

# Add keys to the wallet
[ "$(cleos wallet keys)" == "[]" ] && {
	PK=${EOS_DEVELOPEMENT_KEY}
	run_command "Importing development key" \
	  cleos wallet import -n ${WALLET_NAME} --private-key=$PK

	run_command "Creating keys" \
	  cleos create key --file ${EOS_KEYS_DATA}

	PK=$(head -1 ${EOS_KEYS_DATA} |sed 's/Private key: //')
	run_command "Importing private key" \
	  cleos wallet import -n ${WALLET_NAME} --private-key=$PK
}

# Creating DendrETH account
cleos get account ${DENDRETH_ACCOUNT_IN_EOS} > /dev/null 2>&1
[ $? -eq 1 ] && {
	run_command "Creating DendrETH accounts" \
  	acct ${DENDRETH_ACCOUNT_IN_EOS}
}

# Creating Alica and Bob accounts for testing
cleos get account alice > /dev/null 2>&1
[ $? -eq 1 ] && {
	run_command "Creating DendrETH accounts" \
  	acct alice
}
cleos get account bob > /dev/null 2>&1
[ $? -eq 1 ] && {
	run_command "Creating DendrETH accounts" \
  	acct bob
}
# Set account permission to allow deferred transactions
run_command "Deferred transaction support" \
  cleos set account permission --add-code ${DENDRETH_ACCOUNT_IN_EOS} active
