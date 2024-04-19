#!/usr/bin/env bash
source "${BASH_SOURCE%/*}/../../../../scripts/utils/paths.sh"
source "${BASH_SOURCE%/*}/../common.sh"

CIRCOM_DIR="${ROOT}/beacon-light-client/circom"
SNARKJS_DIR="${ROOT}"/vendor/snarkjs
SNARKJS="$SNARKJS_DIR"/cli.js
PHASE1="${BUILD_DIR}"/pot28_final.ptau
CIRCUIT_NAME=light_client_recursive
BUILD_DIR="${CIRCOM_DIR}/build/$CIRCUIT_NAME"
CIRCUIT_DIR="${CIRCOM_DIR}/scripts/${CIRCUIT_NAME}"

git submodule update --init --recursive

look_for_ptau_file

create_build_folder

compile_the_circuit

# compile_cpp_witness

# echo "****VERIFYING WITNESS****"
# start=$(date +%s)
# ./"$CIRCUIT_NAME" ../../../scripts/"$CIRCUIT_NAME"/input_"$CIRCUIT_NAME".json ../witness.wtns
# end=$(date +%s)
# echo "DONE ($((end - start))s)"

# cd ..
# snarkjs wej witness.wtns witness.json

install_snarkjs_packages

echo "****GENERATING ZKEY 0****"
start=$(date +%s)
cd "$BUILD_DIR" || exit
node \
  --trace-gc \
  --trace-gc-ignore-scavenger \
  --max-old-space-size=2048000 \
  --initial-old-space-size=2048000 \
  --no-incremental-marking \
  --max-semi-space-size=1024 \
  --initial-heap-size=2048000 \
  --expose-gc \
  "$SNARKJS_CLI" zkey new "$CIRCUIT_NAME".r1cs "$PHASE1" "$CIRCUIT_NAME"_0.zkey -v >zkey0.outend=$(date +%s)
echo "DONE ($((end - start))s)"

# contribute_to_phase_2_ceremony

# verify_final_key

# export_vkey
