#!/usr/bin/env bash
source "${BASH_SOURCE%/*}/../../../../scripts/utils/paths.sh"
source "${BASH_SOURCE%/*}/../common.sh"

CIRCOM_DIR="${ROOT}/beacon-light-client/circom"
SNARKJS_DIR="${ROOT}"/vendor/snarkjs
SNARKJS="$SNARKJS_DIR"/cli.js
PHASE1="${BUILD_DIR}"/pot28_final.ptau
CIRCUIT_NAME=validator_balances
BUILD_DIR="${CIRCOM_DIR}/build/$CIRCUIT_NAME"
CIRCUIT_DIR="${CIRCOM_DIR}/scripts/${CIRCUIT_NAME}"

git submodule update --init --recursive

look_for_ptau_file

create_build_folder

compile_the_circuit

compile_cpp_witness

echo "****VERIFYING WITNESS****"
start=$(date +%s)
./"$CIRCUIT_NAME" ../../../scripts/"$CIRCUIT_NAME"/"$CIRCUIT_NAME"-input.json ../witness.wtns
end=$(date +%s)
echo "DONE ($((end - start))s)"

cd ..
yarn snarkjs wej witness.wtns witness.json

install_snarkjs_packages

generate_zkey

contribute_to_phase_2_ceremony

verify_final_key

export_vkey

generate_proof_for_sample_input

verify_proof_for_sample_input
