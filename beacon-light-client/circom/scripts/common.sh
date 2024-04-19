#!/usr/bin/env bash

look_for_ptau_file() {
  if [ -f "$PHASE1" ]; then
    echo "Found Phase 1 ptau file"
  else
    echo "No Phase 1 ptau file found. Exiting..."
    exit 1
  fi
}

create_build_folder() {
  if [ ! -d "$BUILD_DIR" ]; then
    echo "No build directory found. Creating build directory..."
    mkdir -p "$BUILD_DIR"
  fi
}
compile_the_circuit() {
  echo "****COMPILING CIRCUIT****"
  start=$(date +%s)
  cd "${CIRCUIT_DIR}" || exit
  #circom "$CIRCUIT_NAME".circom --O0 --c --output "$BUILD_DIR"
  circom "${CIRCUIT_NAME}".circom --O2 --r1cs --sym --c --output "$BUILD_DIR"
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

compile_cpp_witness() {
  echo "****COMPILING C++ WITNESS GENERATION CODE****"
  start=$(date +%s)
  cd "${BUILD_DIR}"/"${CIRCUIT_NAME}"_cpp || exit
  make
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

install_snarkjs_packages() {
  echo "****INSTALLING SNARKJS PACKAGES****"
  start=$(date +%s)
  cd "$SNARKJS_DIR" || exit
  npm install
  end=$(date +%s)
  echo "DONE ($((end - start))s)"

}

generate_zkey() {
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
    "${SNARKJS}" zkey new "${CIRCUIT_NAME}".r1cs "${PHASE1}" "${CIRCUIT_NAME}"_0.zkey -v >zkey0.out
  # groth16 setup
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

contribute_to_phase_2_ceremony() {
  echo "****CONTRIBUTE TO PHASE 2 CEREMONY****"
  start=$(date +%s)
  cd "${BUILD_DIR}" || exit
  node "${SNARKJS}" zkey contribute -verbose "${CIRCUIT_NAME}"_0.zkey "${CIRCUIT_NAME}".zkey -n="First phase2 contribution" -e="some random text 5555" >contribute.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

verify_final_key() {
  echo "****VERIFYING FINAL ZKEY****"
  start=$(date +%s)
  cd "${BUILD_DIR}" || exit
  node --trace-gc \
    --trace-gc-ignore-scavenger \
    --max-old-space-size=2048000 \
    --initial-old-space-size=2048000 \
    --no-incremental-marking \
    --max-semi-space-size=1024 \
    --initial-heap-size=2048000 \
    --expose-gc \
    "${SNARKJS}" zkey verify -verbose "${CIRCUIT_NAME}".r1cs "${PHASE1}" "${CIRCUIT_NAME}".zkey >verify.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

export_vkey() {
  echo "****EXPORTING VKEY****"
  start=$(date +%s)
  cd "${BUILD_DIR}" || exit
  node "${SNARKJS}" zkey export verificationkey "${CIRCUIT_NAME}".zkey vkey.json -v
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

generate_proof_for_sample_input() {
  echo "****GENERATING PROOF FOR SAMPLE INPUT****"
  start=$(date +%s)
  cd "${BUILD_DIR}" || exit
  node "${SNARKJS}" groth16 prove "${CIRCUIT_NAME}".zkey witness.wtns proof.json public.json >proof.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

verify_proof_for_sample_input() {
  echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
  start=$(date +%s)
  cd "${BUILD_DIR}" || exit
  node "${SNARKJS}" groth16 verify vkey.json public.json proof.json -v
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}
