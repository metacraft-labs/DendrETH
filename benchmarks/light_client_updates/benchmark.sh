#!/usr/bin/env bash

source "${GIT_ROOT}/libs/bash/common-utils/common.sh"

# Define the number of proofs to process
number_of_proofs=10

# Define the log files
log_file="benchmark_times.log"
error_log="error_log.log"

look_for_light_client_zkey_file
look_for_light_client_dat_file
cp "${GIT_ROOT}/light_client.dat" ./
cp "${GIT_ROOT}/relay/light_client" ./

build_dir="${GIT_ROOT}/benchmarks/light_client_updates/build"

if [ -d "${build_dir}" ]; then
  rm -r "${build_dir}"
fi
mkdir "${build_dir}"

# Clear previous log files
>"${log_file}"
>"${error_log}"

# Function to log time separately without including errors in the time log
log_time() {
  {
    { time "$@" 1>&3 2>&4; } 2>&1
  } 2>&1 | tee -a "${log_file}" 1>&2
  echo "" >>"${log_file}"
}

# Loop to get the first number_of_proofs proofs
for i in $(seq 1 $number_of_proofs); do
  # Retrieve the data from Redis and format it
  json_data=$(redis-cli HGET bull:proof:"${i}" data | awk -F'{"proofInput":' '{print $2}' | awk -F',"prevUpdateSlot"' '{print $1}')

  # Save the data to a JSON file
  echo "${json_data}" >build/proof_"${i}".json

  # Log the iteration number
  echo "Processing proof ${i}" | tee -a "${log_file}"

  # Time the creation of the witness
  echo "Witness creation time:" >>"${log_file}"
  log_time ./light_client build/proof_"${i}".json build/witness_"${i}".wtns 3>>"${log_file}" 4>>"${error_log}"

  # Time the prover step
  echo "Prover time:" >>"${log_file}"
  log_time prover ./../../data/light_client.zkey build/witness_"${i}".wtns build/proof_"${i}".json build/public_"${i}".json 3>>"${log_file}" 4>>"${error_log}"

  echo "Finished processing proof ${i}" >>"${log_file}"
  echo "---------------------------------" >>"${log_file}"

  # Print status to console
  echo "Completed proof ${i}/${number_of_proofs}"
done

echo "Proofs have been saved to JSON files and timing information has been logged."
echo "Errors and warnings have been logged to ${error_log}"
