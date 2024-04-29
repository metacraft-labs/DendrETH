#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/../../scripts/utils/paths.sh"
source "${ROOT}/.env"

if [[ -z "${PROVER_SERVER_HOST}" ]] && [[ -z "${PROVER_SERVER_PORT}" ]]; then
  PROVER_SERVER_HOST="http://127.0.0.1"
  PROVER_SERVER_PORT="5000"
fi

check_proverserver() {
  netstat -tuln | grep ":${PROVER_SERVER_PORT}" >/dev/null
}

while ! check_proverserver; do
  echo "ProverServer is not running on port ${PROVER_SERVER_PORT}. Waiting..."
  sleep 5
done

echo "ProverServer is up and running on port ${PROVER_SERVER_PORT}!"
