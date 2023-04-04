#!/usr/bin/env bash

ZKEY_SH256_SUM='2073fef22678def027a69c075e4ca4ace68461d99f545f55360601660eb30f4b'

cd DendrETH

if [[ -z "$WITNESS_GENERATOR_PATH" ]]; then
  echo "WITNESS_GENERATOR_PATH environment variable is not set. Using default value."
  WITNESS_GENERATOR_PATH="/DendrETH/vendor/build-artifacts/light_client_cpp/light_client"
else
  echo "Using WITNESS_GENERATOR_PATH=$WITNESS_GENERATOR_PATH"
fi

if [[ -z "$RAPIDSNAKR_PROVER_PATH" ]]; then
  echo "RAPIDSNAKR_PROVER_PATH environment variable is not set. Using default value."
  RAPIDSNAKR_PROVER_PATH="/DendrETH/vendor/rapidsnark/build/prover"
else
  echo "Using RAPIDSNAKR_PROVER_PATH=$RAPIDSNAKR_PROVER_PATH"
fi

if [ ! -z "$ZKEY_FILE_PATH" ]; then
  echo "ZKEY_FILE_PATH environment variable is not set. Using default value."
  ZKEY_FILE_PATH="/DendrETH/build/light_client.zkey"
else
  echo "Using ZKEY_FILE_PATH=$ZKEY_FILE_PATH"
fi

if [ ! -f "$ZKEY_FILE_PATH" ]; then
  echo "Downloading zkey file from http://dendreth.metacraft-labs.com/capella_74.zkey ..."
  curl http://dendreth.metacraft-labs.com/capella_74.zkey > "$ZKEY_FILE_PATH" && \
  echo "$ZKEY_SH256_SUM $ZKEY_FILE_PATH" | sha256sum -c
  if [ $? -eq 0 ]; then
    echo "Zkey file downloaded successfully to $ZKEY_FILE_PATH"
  else
    echo "Failed to download zkey file from http://dendreth.metacraft-labs.com/capella_74.zkey"
    exit 1
  fi
else
  echo "Using cached zkey file at $ZKEY_FILE_PATH"
fi

if [[ -z "$REDIS_HOST" ]] && [[ -z "$REDIS_PORT" ]]; then
  echo "REDIS_HOST and REDIS_PORT environment variables are not set. Using default values."
  REDIS_HOST="localhost"
  REDIS_PORT="6379"
else
  echo "Using Redis settings from environment variables"
fi

nix --experimental-features 'nix-command flakes' --accept-flake-config develop .#devShells.x86_64-linux.container --command bash -c '
# needed in order for the supervisord configuration to be correct
mkdir -p redis-server

supervisord -c supervisord.conf

if [[ "$REDIS_HOST" == "localhost" ]] && [[ "$REDIS_PORT" == "6379" ]]; then
  echo "Starting local Redis server..."
  supervisorctl start redis
  echo "Local Redis server started"
else
  echo "Using remote Redis server at $REDIS_HOST:$REDIS_PORT"
fi

# Run the polling update task
echo "Starting the polling update task"
supervisorctl start pollUpdatesWorker
echo "Polling update task started"

# Run the proof generation task
echo "Starting the proof generation task"
supervisorctl start proofGenerationWorker
echo "Proof generation task started"

# compile contracts
cd beacon-light-client/solidity
yarn hardhat compile
cd ../../

if [ -z "$INITIAL_SLOT" ]; then
  echo "Error: INITIAL_SLOT environment variable is not set. Exiting..."
  exit 1
fi

if [ -z "$SLOTS_JUMP" ]; then
  echo "Error: SLOTS_JUMP environment variable is not set. Exiting..."
  exit 1
fi

# Register update polling task
supervisorctl start runUpdate
echo "Registered update polling repeat task"

# Run hardhat tasks for different networks
if [ -n "$LC_GOERLI" ]; then
  echo "Starting light client for Goerli network"
  supervisorctl start goerli
else
  echo "Skipping Goerli network"
fi

if [ -n "$LC_OPTIMISTIC_GOERLI" ]; then
  echo "Starting light client for Optimistic Goerli network"
  supervisorctl start optimisticGoerli
else
  echo "Skipping Optimistic Goerli network"
fi

if [ -n "$LC_BASE_GOERLI" ]; then
  echo "Starting light client for Base Goerli network"
  supervisorctl start baseGoerli
else
  echo "Skipping Base Goerli network"
fi

if [ -n "$LC_ARBITRUM_GOERLI" ]; then
  echo "Starting light client for Arbitrum Goerli network"
  supervisorctl start arbitrumGoerli
else
  echo "Skipping Arbitrum Goerli network"
fi

if [ -n "$LC_SEPOLIA" ]; then
  echo "Starting light client for Sepolia network"
  supervisorctl start sepolia
else
  echo "Skipping Sepolia network"
fi

if [ -n "$LC_MUMBAI" ]; then
  echo "Starting light client for Mumbai network"
  supervisorctl start mumbai
else
  echo "Skipping Mumbai network"
fi

echo "Everything started"

supervisorctl start general_logs

tail -f relay/general_logs.log relay/pollUpdatesWorker.log relay/proofGenerationWorker.log beacon-light-client/solidity/goerli.log beacon-light-client/solidity/optimisticGoerli.log beacon-light-client/solidity/baseGoerli.log beacon-light-client/solidity/arbitrumGoerli.log beacon-light-client/solidity/sepolia.log beacon-light-client/solidity/mumbai.log

'
