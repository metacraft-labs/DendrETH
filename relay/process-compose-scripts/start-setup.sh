#!/usr/bin/env bash

source "${GIT_ROOT}/.env"
source "${GIT_ROOT}/libs/bash/common-utils/common.sh"

(cd "${GIT_ROOT}/beacon-light-client/solidity/" && yarn hardhat compile)

git submodule update --init --recursive

look_for_light_client_zkey_file
look_for_light_client_dat_file

# rapidnskark prover server searches for the witness generator exe in build directory
mkdir -p "${GIT_ROOT}/build"
cp "${GIT_ROOT}/relay/light_client" "${GIT_ROOT}/build/light_client"
cp "${GIT_ROOT}/data/light_client.dat" "${GIT_ROOT}/light_client.dat"

if [ -z "${SLOTS_JUMP}" ]; then
  echo "Error: SLOTS_JUMP environment variable is not set. Exiting..."
  exit 1
fi

if [[ "${PRATTER}" != "TRUE" && "${MAINNET}" != "TRUE" ]]; then
  echo "Neither PRATTER nor MAINNET is set or true."
  exit 1
fi
