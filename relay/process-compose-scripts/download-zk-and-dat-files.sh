#!/usr/bin/env bash

ZKEY_B3SUM_SUM='7bd1baf6e4aa1bb97933df06f68b26f8aa034e6743ff52c4dd7f6097d6e7d104'
DAT_B3SUM_SUM='c94eb86af7c0451a4277a7bdfc90232a9db75c192d6852ad18baa9a46e1e52e5'

source "${GIT_ROOT}/.env"

cd "${GIT_ROOT}/beacon-light-client/solidity/" && yarn hardhat compile
cd ../..

git submodule update --init --recursive

calculate_checksum() {
  local FILE_PATH=$1
  b3sum "${FILE_PATH}" | cut -d ' ' -f 1
}

download_zkey_file() {

if [[ -z "${LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION}" ]]; then
  echo "Light_Client_ZKEY_DOWNLOAD_LOCATION  environment variables are not set. Using default values."
  LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION="https://dendrethstorage.blob.core.windows.net/light-client/light-client.zkey"
  echo "This might take a while as the file is 52G"
else
  echo "Using download zkey settings from environment variables"
fi

  echo "Downloading zkey file from "${LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION}" ..."

  curl "${LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION}" >"data/light_client.zkey"

  CALCULATED_ZKEY_SUM=$(calculate_checksum data/light_client.zkey)

  if [ "${ZKEY_B3SUM_SUM}" = "${CALCULATED_ZKEY_SUM}" ]; then
    echo "Zkey file downloaded successfully to data/light_client.zkey"
  else
    echo "Failed to download zkey file from "${LIGHT_CLIENT_ZKEY_DOWNLOAD_LOCATION}
    exit 1
  fi
}

download_dat_file() {

if [[ -z "${LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION}" ]]; then
  echo "LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION  environment variables are not set. Using default values."
  LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION="https://dendrethstorage.blob.core.windows.net/light-client/light-client.dat"
else
  echo "Using download dat settings from environment variables"
fi

  echo "Downloading .dat file from "${LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION}" ..."

  curl -k "${LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION}" >"data/light_client.dat"

  CALCULATED_DAT_SUM=$(calculate_checksum data/light_client.dat)

  if [ "${DAT_B3SUM_SUM}" = "${CALCULATED_DAT_SUM}" ]; then
    echo ".dat file downloaded successfully to data/light_client.dat"
  else
    echo "Failed to download .dat file from "${LIGHT_CLIENT_DAT_DOWNLOAD_LOCATION}""
    exit 1
  fi
}

mkdir -p "${GIT_ROOT}/data"

if [ ! -f "${GIT_ROOT}/data/light_client.zkey" ]; then
  download_zkey_file
else
  CALCULATED_ZKEY_SUM=$(calculate_checksum "${GIT_ROOT}/data/light_client.zkey")
  echo "${CALCULATED_ZKEY_SUM}"
  if [ "${ZKEY_B3SUM_SUM}" = "${CALCULATED_ZKEY_SUM}" ]; then
    echo "Using cached zkey file at ${GIT_ROOT}/data/light_client.zkey"
  else
    echo "Wrong version of light_client.zkey cached downloading again..."
    download_zkey_file
  fi
fi

if [ ! -f "${GIT_ROOT}/data/light_client.dat" ]; then
  download_dat_file
else
  CALCULATED_DAT_SUM=$(calculate_checksum "${GIT_ROOT}/data/light_client.dat")
  echo "${CALCULATED_DAT_SUM}"
  if [ "${DAT_B3SUM_SUM}" = "${CALCULATED_DAT_SUM}" ]; then
    echo "Using cached .dat file at ${GIT_ROOT}/data/light_client.dat"
  else
    echo "Wrong version of light_client.dat cached downloading again..."
    download_dat_file
  fi
fi

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
