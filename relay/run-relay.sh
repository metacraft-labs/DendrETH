#!/bin/bash

ZKEY_B3SUM_SUM='b36a78df185bc2da310ac6cd9451f143fd13461d059369184d93191972850746'
DAT_B3SUM_SUM='dc18f6de3f7905964f467e3626773e6960199273706c2df63bf5991091a74032'
source .env
cd beacon-light-client/solidity/ && yarn hardhat compile
cd ../..
process-compose -t=false &

calculate_checksum() {
  local FILE_PATH=$1
  b3sum "${FILE_PATH}" | cut -d ' ' -f 1
}

download_zkey_file() {
  echo "Downloading zkey file from http://dendreth.metacraft-labs.com/capella_94.zkey ..."

  curl http://dendreth.metacraft-labs.com/capella_94.zkey >"data/light_client.zkey"

  CALCULATED_ZKEY_SUM=$(calculate_checksum data/light_client.zkey)

  if [ "${ZKEY_B3SUM_SUM}" = "${CALCULATED_ZKEY_SUM}" ]; then
    echo "Zkey file downloaded successfully to data/light_client.zkey"
  else
    echo "Failed to download zkey file from http://dendreth.metacraft-labs.com/capella_94.zkey"
    exit 1
  fi
}

if [ "${MOCK}" == FALSE ]; then
  download_dat_file() {
    echo "Downloading .dat file from https://media.githubusercontent.com/media/metacraft-labs/DendrETH-build-artifacts/master/light_client_cpp/light_client.dat ..."

    curl -k https://media.githubusercontent.com/media/metacraft-labs/DendrETH-build-artifacts/master/light_client_cpp/light_client.dat >"data/light_client.dat"

    CALCULATED_DAT_SUM=$(calculate_checksum data/light_client.dat)

    if [ "${DAT_B3SUM_SUM}" = "${CALCULATED_DAT_SUM}" ]; then
      echo ".dat file downloaded successfully to data/light_client.dat"
    else
      echo "Failed to download .dat file from https://media.githubusercontent.com/media/metacraft-labs/DendrETH-build-artifacts/master/light_client_cpp/light_client.dat"
      exit 1
    fi
  }

  if [ ! -f "data/light_client.zkey" ]; then
    download_zkey_file
  else
    CALCULATED_ZKEY_SUM=$(calculate_checksum data/light_client.zkey)
    echo "${CALCULATED_ZKEY_SUM}"
    if [ "${ZKEY_B3SUM_SUM}" = "${CALCULATED_ZKEY_SUM}" ]; then
      echo "Using cached zkey file at data/light_client.zkey"
    else
      echo "Wrong version of light_client.zkey cached downloading again..."
      download_zkey_file
    fi
  fi

  if [ ! -f "data/light_client.dat" ]; then
    download_dat_file
  else
    CALCULATED_DAT_SUM=$(calculate_checksum data/light_client.dat)
    echo "${CALCULATED_DAT_SUM}"
    if [ "${DAT_B3SUM_SUM}" = "${CALCULATED_DAT_SUM}" ]; then
      echo "Using cached .dat file at data/light_client.dat"
    else
      echo "Wrong version of light_client.dat cached downloading again..."
      download_dat_file
    fi
  fi

else
  echo "Mock mode, not downloading .zkey and .dat files"
fi

# rapidnskark prover server searches for the witness generator exe in build directory
mkdir -p build
cp relay/light_client build/light_client
cp data/light_client.dat light_client.dat

# if [[ -z "${REDIS_HOST}" ]] && [[ -z "${REDIS_PORT}" ]]; then
echo "REDIS_HOST and REDIS_PORT environment variables are not set. Using default values."
REDIS_HOST="localhost"
REDIS_PORT="6379"
# else
# echo "Using Redis settings from environment variables"
# fi

if [[ -z "${PROVER_SERVER_HOST}" ]] && [[ -z "${PROVER_SERVER_PORT}" ]]; then
  echo "PROVER_SERVER_HOST and PROVER_SERVER_PORT environment variables are not set. Using default values."
  PROVER_SERVER_HOST="http://127.0.0.1"
  PROVER_SERVER_PORT="5000"
else
  echo "Using prover server settings from environment variables"
fi

if [[ "${MOCK}" == FALSE ]]; then
  if [[ "${PROVER_SERVER_HOST}" == "http://127.0.0.1" ]]; then
    echo "Starting local prover server..."
    process-compose process start proverserver
    echo "Prover server started with command"

    max_attempts=300 # 300 attempts * 2s delay = 10 minutes
    server_started=false

    echo "Waiting for server to start..."

    for ((i = 1; i <= max_attempts; i++)); do
      response=$(curl -s -o /dev/null -w "%{http_code}" "${PROVER_SERVER_HOST}":"${PROVER_SERVER_PORT}"/status)

      if [ "${response}" -eq 200 ]; then
        echo "Server is up and running."
        server_started=true
        break
      fi

      echo "Attempt ${i}: Server is not responding. Waiting for 2 seconds..."
      sleep 2
    done

    if [ ${server_started} == false ]; then
      echo "Server failed to start after 5 minutes. Exiting."
      exit 1
    fi
  else
    echo "Using remote prover server at ${PROVER_SERVER_HOST}:${PROVER_SERVER_PORT}"
  fi
else
  echo "Mock mode, not starting local prover server"
fi

# if [["${MOCK}" == FALSE ]]; then
if [[ "${REDIS_HOST}" == "localhost" ]] && [[ "${REDIS_PORT}" == "6379" ]]; then
  echo "Starting local Redis server..."
  process-compose process start redis
  echo "Local Redis server started"
else
  echo "Using remote Redis server at ${REDIS_HOST}:${REDIS_PORT}"
fi

echo "Starting Prometheus server on 9090"
process-compose process start prometheus
echo "Prometheus server started"

# Run the polling update task
echo "Starting the polling update task"
process-compose process start pollUpdatesWorker
echo "Polling update task started"

# Run the proof generation task
echo "Starting the proof generation task"
process-compose process start proofGenerationWorker
echo "Proof generation task started"

if [ -z "${SLOTS_JUMP}" ]; then
  echo "Error: SLOTS_JUMP environment variable is not set. Exiting..."
  exit 1
fi

if [[ "${PRATTER}" != "TRUE" && "${MAINNET}" != "TRUE" ]]; then
  echo "Neither PRATTER nor MAINNET is set or true."
  exit 1
fi

run_network_tasks() {

  # Run hardhat tasks for different networks
  if [ -n "${LC_GOERLI}" ]; then
    echo "Starting light client for Goerli network"
    process-compose process start goerli
  else
    echo "Skipping Goerli network"
  fi
  if [[ "${MOCK}" == FALSE ]]; then
    if [ -n "${LC_OPTIMISTIC_GOERLI}" ]; then
      echo "Starting light client for Optimistic Goerli network"
      process-compose process start goeoptimisticGoerlirli
    else
      echo "Skipping Optimistic Goerli network"
    fi

    if [ -n "${LC_BASE_GOERLI}" ]; then
      echo "Starting light client for Base Goerli network"
      process-compose process start baseGoerli
    else
      echo "Skipping Base Goerli network"
    fi

    if [ -n "${LC_ARBITRUM_GOERLI}" ]; then
      echo "Starting light client for Arbitrum Goerli network"
      process-compose process start arbitrumGoerli
    else
      echo "Skipping Arbitrum Goerli network"
    fi

    if [ -n "${LC_SEPOLIA}" ]; then
      echo "Starting light client for Sepolia network"
      process-compose process start sepolia
    else
      echo "Skipping Sepolia network"
    fi

    if [ -n "${LC_MUMBAI}" ]; then
      echo "Starting light client for Mumbai network"
      process-compose process start mumbai
    else
      echo "Skipping Mumbai network"
    fi

    if [ -n "${LC_FUJI}" ]; then
      echo "Starting light client for Fuji network"
      process-compose process start fuji
    else
      echo "Skipping Fuji network"
    fi

    if [ -n "${LC_FANTOM}" ]; then
      echo "Starting light client for Fantom network"
      process-compose process start fantom
    else
      echo "Skipping Fantom network"
    fi

    if [ -n "${LC_ALFAJORES}" ]; then
      echo "Starting light client for Alfajores network"
      process-compose process start alfajores
    else
      echo "Skipping Alfajores network"
    fi

    if [ -n "${LC_BSC}" ]; then
      echo "Starting light client for BSC network"
      process-compose process start bsc
    else
      echo "Skipping BSC network"
    fi

    if [ -n "${LC_AURORA}" ]; then
      echo "Starting light client for Aurora network"
      process-compose process start aurora
    else
      echo "Skipping Aurora network"
    fi

    if [ -n "${LC_GNOSIS}" ]; then
      echo "Starting light client for Gnosis network"
      process-compose process start gnosis
    else
      echo "Skipping Gnosis network"
    fi

    if [ -n "${LC_CHIADO}" ]; then
      echo "Starting light client for Chiado network"
      process-compose process start chiado
    else
      echo "Skipping Chiado network"
    fi

    if [ -n "${LC_EVMOS}" ]; then
      echo "Starting light client for EVMOS network"
      process-compose process start evmos
    else
      echo "Skipping EVMOS network"
    fi

    if [ -n "${LC_MALAGA}" ]; then
      echo "Starting light client for Malaga network"
      process-compose process start malaga
    else
      echo "Skipping Malaga network"
    fi
  else
    echo "Mock mode, skipped all but 1 network tasks"
  fi
  echo "Everything started for ${FOLLOW_NETWORK}"
}

# Call the function based on PRATTER or MAINNET
if [[ "${PRATTER}" == "TRUE" ]]; then
  export FOLLOW_NETWORK="pratter"
  run_network_tasks
fi

if [[ "${MAINNET}" == "TRUE" ]]; then
  export FOLLOW_NETWORK="mainnet"
  run_network_tasks
fi

process-compose process start cleaner

process-compose process start general_logs

echo "____________Starting Logs____________"

tail -f .logs/prover_server.log logs/general_logs.log logs/pollUpdatesWorker.log logs/proofGenerationWorker.log logs/goerli.log logs/optimisticGoerli.log logs/baseGoerli.log logs/arbitrumGoerli.log logs/sepolia.log logs/mumbai.log logs/gnosis.log
