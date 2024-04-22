#!/usr/bin/env bash
source "${BASH_SOURCE%/*}/../../../../scripts/utils/paths.sh"
source "${BASH_SOURCE%/*}/../common.sh"

CIRCOM_DIR="${ROOT}/beacon-light-client/circom"
SNARKJS_DIR="${ROOT}"/vendor/snarkjs
SNARKJS="${SNARKJS_DIR}"/cli.js
PHASE1="${BUILD_DIR}"/pot28_final.ptau
CIRCUIT_NAME=light_client
BUILD_DIR="${CIRCOM_DIR}/build/${CIRCUIT_NAME}"
CIRCUIT_DIR="${CIRCOM_DIR}/scripts/${CIRCUIT_NAME}"

git submodule update --init --recursive

look_for_ptau_file

create_build_folder

compile_the_circuit

compile_cpp_witness

install_snarkjs_packages

generate_zkey

export_vkey
