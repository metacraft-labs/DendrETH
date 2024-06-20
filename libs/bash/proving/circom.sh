#!/usr/bin/env bash

export CIRCOM_DIR="${GIT_ROOT}/beacon-light-client/circom"
export SNARKJS_DIR="${GIT_ROOT}/vendor/snarkjs"
export SNARKJS="${SNARKJS_DIR}/cli.js"

compile_the_circuit() {
  local circuit_dir="$1"
  local circuit_name="$2"
  local build_dir="$3"

  echo "****COMPILING CIRCUIT****"
  start=$(date +%s)
  cd "${circuit_dir}" || exit
  #circom "${circuit_name}".circom --O0 --c --output "${build_folder}"
  circom "${circuit_name}.circom" --O2 --r1cs --sym --c --output "${build_dir}"
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

compile_cpp_witness() {
  local build_dir="$1"
  local circuit_name="$2"

  echo "****COMPILING C++ WITNESS GENERATION CODE****"
  start=$(date +%s)
  cd "${build_dir}/${circuit_name}_cpp" || exit
  make
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

verify_witness() {
  local build_dir="$1"
  local circuit_name="$2"
  local input_json="$3"
  local snarkjs="$4"

  echo "****VERIFYING WITNESS****"
  start=$(date +%s)
  cd "${build_dir}/${circuit_name}_cpp" || exit
  ./"${circuit_name}" "${input_json}" ../witness.wtns
  end=$(date +%s)
  echo "DONE ($((end - start))s)"

  node "${snarkjs}" wej "${build_dir}/witness.wtns" "${build_dir}/witness.json"
}

install_snarkjs_packages() {
  local snarkjs_dir="$1"

  echo "****INSTALLING SNARKJS PACKAGES****"
  start=$(date +%s)
  cd "${snarkjs_dir}" || exit
  npm install
  end=$(date +%s)
  echo "DONE ($((end - start))s)"

}

generate_zkey() {
  local build_dir="$1"
  local snarkjs="$2"
  local phase1_file="$3"
  local circuit_name="$4"

  echo "****GENERATING ZKEY 0****"
  start=$(date +%s)
  cd "${build_dir}" || exit
  node \
    --trace-gc \
    --trace-gc-ignore-scavenger \
    --max-old-space-size=2048000 \
    --initial-old-space-size=2048000 \
    --no-incremental-marking \
    --max-semi-space-size=1024 \
    --initial-heap-size=2048000 \
    --expose-gc \
    "${snarkjs}" zkey new "${circuit_name}".r1cs "${phase1_file}" "${circuit_name}".zkey -v >zkey0.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

# contribute_to_phase_2_ceremony() {
#   local build_dir="$1"
#   local snarkjs="$2"
#   local circuit_name="$3"

#   echo "****CONTRIBUTE TO PHASE 2 CEREMONY****"
#   start=$(date +%s)
#   cd "${build_dir}" || exit
#   node "${snarkjs}" zkey contribute -verbose "${circuit_name}"_0.zkey "${circuit_name}".zkey -n="First phase2 contribution" -e="some random text 5555" >contribute.out
#   end=$(date +%s)
#   echo "DONE ($((end - start))s)"
# }

verify_final_key() {
  local build_dir="$1"
  local snarkjs="$2"
  local circuit_name="$3"
  local phase1_file="$4"

  echo "****VERIFYING FINAL ZKEY****"
  start=$(date +%s)
  cd "${build_dir}" || exit
  node --trace-gc \
    --trace-gc-ignore-scavenger \
    --max-old-space-size=2048000 \
    --initial-old-space-size=2048000 \
    --no-incremental-marking \
    --max-semi-space-size=1024 \
    --initial-heap-size=2048000 \
    --expose-gc \
    "${snarkjs}" zkey verify -verbose "${circuit_name}".r1cs "${phase1_file}" "${circuit_name}".zkey >verify.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

export_vkey() {
  local build_dir="$1"
  local snarkjs="$2"
  local circuit_name="$3"

  echo "****EXPORTING VKEY****"
  start=$(date +%s)
  cd "${build_dir}" || exit
  node "${snarkjs}" zkey export verificationkey "${circuit_name}".zkey vkey.json -v
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

generate_proof_for_sample_input() {
  local build_dir="$1"
  local snarkjs="$2"
  local circuit_name="$3"

  echo "****GENERATING PROOF FOR SAMPLE INPUT****"
  start=$(date +%s)
  cd "${build_dir}" || exit
  node "${snarkjs}" groth16 prove "${circuit_name}".zkey witness.wtns proof.json public.json >proof.out
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}

verify_proof_for_sample_input() {
  local build_dir="$1"
  local snarkjs="$2"

  echo "****VERIFYING PROOF FOR SAMPLE INPUT****"
  start=$(date +%s)
  cd "${build_dir}" || exit
  node "${snarkjs}" groth16 verify vkey.json public.json proof.json -v
  end=$(date +%s)
  echo "DONE ($((end - start))s)"
}
