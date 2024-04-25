#!/usr/bin/env bash

set -euo pipefail

source "${BASH_SOURCE%/*}/../../../../scripts/utils/paths.sh"
source "${BASH_SOURCE%/*}/../common.sh"

CIRCUIT_NAME="compress"
BUILD_DIR="${CIRCOM_DIR}/build/${CIRCUIT_NAME}"
CIRCUIT_DIR="${CIRCOM_DIR}/scripts/${CIRCUIT_NAME}"
INPUT_JSON="${CIRCOM_DIR}/test/${CIRCUIT_NAME}/data/case01/input.json"
PHASE1="${BUILD_DIR}/pot28_final.ptau"

git submodule update --init --recursive

# ****CHECK FOR PTAU FILE****
(look_for_ptau_file "${PHASE1}")

# ****CHECK FOR BUILD FOLDER****
(create_build_folder "${BUILD_DIR}")

# ****MAKE SURE WE HAVE CORRECT SNARKJS****
(install_snarkjs_packages "${SNARKJS_DIR}")

# ****COMPILING CIRCUIT****
(compile_the_circuit "${CIRCUIT_DIR}" "${CIRCUIT_NAME}" "${BUILD_DIR}")

# ****COMPILING C++ WITNESS GENERATION CODE****
(compile_cpp_witness "${BUILD_DIR}" "${CIRCUIT_NAME}")

# ****CREATE WITNESS FROM INPUT DATA****
(verify_witness "${BUILD_DIR}" "${CIRCUIT_NAME}" "${INPUT_JSON}" "${SNARKJS}")

# ****GENERATE ZKEY****
(generate_zkey "${BUILD_DIR}" "${SNARKJS}" "${PHASE1}" "${CIRCUIT_NAME}")

# ****CREATE FINAL ZKEY****
(verify_final_key "${BUILD_DIR}" "${SNARKJS}" "${CIRCUIT_NAME}" "${PHASE1}")

# ****EXPORT ZKEY TO JSON****
(export_vkey "${BUILD_DIR}" "${SNARKJS}" "${CIRCUIT_NAME}")

# ****CREATE PROOF FOR THE WITNESS WE CREATED ABOVE****
(generate_proof_for_sample_input "${BUILD_DIR}" "${SNARKJS}" "${CIRCUIT_NAME}")

# ****CHECK IF PROOF IS CORRECT****
(verify_proof_for_sample_input "${BUILD_DIR}" "${SNARKJS}")
