#!/usr/bin/env bash

DENDRETH_DIR=$(git rev-parse --show-toplevel)
DENDRETH_ACCOUNT_IN_EOS="dendreth"

EOS_DEVELOPEMENT_KEY=5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3
EOS_CONTRACT_DIR=${DENDRETH_DIR}/contracts/eos
EOS_NODE_DATA_DIR_DIR=${EOS_CONTRACT_DIR}/node-data-dir
EOS_WALLET_DATA=${EOS_NODE_DATA_DIR_DIR}/.wallet
EOS_KEYS_DATA=${EOS_NODE_DATA_DIR_DIR}/.keys
EOS_LOGS_DIR=${EOS_NODE_DATA_DIR_DIR}/logs

WALLET_NAME="DendrETH-wallet"

CMD=$1; shift

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
  [ -d ${EOS_NODE_DATA_DIR_DIR} ] && rm -rf ${EOS_NODE_DATA_DIR_DIR}
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
    -e -p eosio                                      \
    --data-dir ${EOS_NODE_DATA_DIR_DIR}                  \
    --config-dir ${EOS_NODE_DATA_DIR_DIR}                \
    --plugin eosio::producer_plugin                  \
    --plugin eosio::chain_plugin                     \
    --plugin eosio::http_plugin                      \
    --plugin eosio::state_history_plugin             \
    --plugin eosio::chain_api_plugin                 \
    --contracts-console                              \
    --disable-replay-opts                            \
    --access-control-allow-origin='*'                \
    --http-validate-host=false                       \
    --verbose-http-errors                            \
    --state-history-dir ${EOS_NODE_DATA_DIR_DIR}/shpdata \
    --trace-history                                  \
    --chain-state-history                            \
    --replay-blockchain > ${EOS_LOGS_DIR}/nodeos.log 2>&1 &
}

# ------------------------------------------------------------------------------
[ "$CMD" == "stop" ] && {
	echo "* Stopping nodeos..."
	quit "nodeos"
	echo "* Stopping keosd..."
	quit "keosd"
  echo "* Cleaning up..."
  cleanup
	exit 0
}

[ "$CMD" == "start" ] && {
	start_$1
	exit 0
}

[ $(isRunning keosd) == "N" ] && {
	echo "* Starting keosd..."
	start_keosd
}
[ $(isRunning nodeos) == "N" ] && {
	echo "* Starting nodeos..."
	start_nodeos
	sleep 1
}

# Create the wallet if it doesn't exist
[ $(walletExists) == "N" ] && {
    echo "Creating wallet..."
    cleos wallet create -n ${WALLET_NAME} --file ${EOS_WALLET_DATA}
}

# Unlock the wallet
walletUnlock

# Add keys to the wallet
[ "$(cleos wallet keys)" == "[]" ] && {
	echo "* Importing development key..."
	PK=${EOS_DEVELOPEMENT_KEY}
	cleos wallet import -n ${WALLET_NAME} --private-key=$PK

	echo "* Creating keys..."
	cleos create key --file ${EOS_KEYS_DATA}

	echo "* Importing private key..."
	PK=$(head -1 ${EOS_KEYS_DATA} |sed 's/Private key: //')
	cleos wallet import -n ${WALLET_NAME} --private-key=$PK
}

# Creating system account
cleos get account ${DENDRETH_ACCOUNT_IN_EOS} > /dev/null 2>&1
[ $? -eq 1 ] && {
	echo "* Creating system accounts..."
	acct ${DENDRETH_ACCOUNT_IN_EOS}
}

# Set account permission to allow deferred transactions
echo "* Deferred transaction support..."
cleos set account permission --add-code ${DENDRETH_ACCOUNT_IN_EOS} active
