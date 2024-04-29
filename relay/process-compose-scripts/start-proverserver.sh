#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/../../scripts/utils/paths.sh"
source "${ROOT}/.env"

if [[ -z "${PROVER_SERVER_HOST}" ]] && [[ -z "${PROVER_SERVER_PORT}" ]]; then
  echo "PROVER_SERVER_HOST and PROVER_SERVER_PORT environment variables are not set. Using default values."
  PROVER_SERVER_HOST="http://127.0.0.1"
  PROVER_SERVER_PORT="5000"
else
  echo "Using prover server settings from environment variables"
fi

if [[ "${PROVER_SERVER_HOST}" == "http://127.0.0.1" ]]; then
  (
    cd "${ROOT}" || exit 1
    proverServer "${PROVER_SERVER_PORT}" "${ROOT}/data/light_client.zkey"
    echo "Prover server started with command"
  )

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
