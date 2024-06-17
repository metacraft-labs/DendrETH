#!/usr/bin/env bash

source "${GIT_ROOT}/.env"

# Define the base directory for the output files (2 folders back)
output_dir="${GIT_ROOT}/proofs"

# Create the base directory if it doesn't exist
mkdir -p "${output_dir}"

# Function to convert binary string to hexadecimal
bin_to_hex() {
  local bin_string=$1
  local hex_string=""
  local decimal_value=0

  # Iterate through each binary digit in chunks of 4
  for ((i = 0; i < ${#bin_string}; i += 4)); do
    # Extract 4-bit chunk
    chunk="${bin_string:$i:4}"
    # Convert binary chunk to decimal
    decimal_value=$((2#${chunk}))
    # Append hexadecimal representation to result
    hex_string+=$(printf "%X" ${decimal_value})
  done

  echo "0x${hex_string}"
}

# Get the first 10 keys matching the pattern proof:*
keys=$(redis-cli --scan --pattern 'proof:*' | head -n 10)

for key in ${keys}; do
  # Retrieve the JSON data from Redis
  json_data=$(redis-cli GET "${key}")

  # Check if data is not empty
  if [ -n "${json_data}" ]; then
    echo "Processing ${key}"

    # Extract the parts of the key
    IFS=':' read -r -a key_parts <<<"${key}"
    n1=${key_parts[1]}
    n2=${key_parts[2]}

    # Use jq to extract pi_a, pi_b, pi_c, and public
    extracted_proof=$(echo "${json_data}" | jq '.proof | {pi_a, pi_b, pi_c}')
    extracted_public=$(echo "${json_data}" | jq '.public')

    # Extract update data from proofInput
    nextHeaderHash_bin=$(echo "${json_data}" | jq -r '.proofInput.nextHeaderHash // empty | map(.) | join("")')
    finalizedHeaderRoot_bin=$(echo "${json_data}" | jq -r '.proofInput.finalizedHeaderRoot // empty | map(.) | join("")')
    execution_state_root_bin=$(echo "${json_data}" | jq -r '.proofInput.execution_state_root // empty | map(.) | join("")')

    nextHeaderHash_hex=$(bin_to_hex "${nextHeaderHash_bin}")
    finalizedHeaderRoot_hex=$(bin_to_hex "${finalizedHeaderRoot_bin}")
    execution_state_root_hex=$(bin_to_hex "${execution_state_root_bin}")

    # Extract nextHeaderSlot
    nextHeaderSlot=$(echo "${json_data}" | jq -r '.proofInput.nextHeaderSlot // empty')

    # Construct file paths
    proof_output_filename="${output_dir}/proof_${n1}_${n2}.json"
    public_output_filename="${output_dir}/public_${n1}_${n2}.json"
    update_output_filename="${output_dir}/update_${n1}_${n2}.json"

    # Save extracted proof data to JSON file
    echo "${extracted_proof}" >"${proof_output_filename}"

    # Save extracted public data to JSON file
    echo "${extracted_public}" >"${public_output_filename}"

    # Save converted fields to update JSON file
    echo "{
  \"attestedHeaderRoot\": \"${nextHeaderHash_hex}\",
  \"attestedHeaderSlot\": ${nextHeaderSlot},
  \"finalizedHeaderRoot\": \"${finalizedHeaderRoot_hex}\",
  \"finalizedExecutionStateRoot\": \"${execution_state_root_hex}\"
}" >"${update_output_filename}"

    echo "Saved to ${proof_output_filename}, ${public_output_filename}, and ${update_output_filename}"
  else
    echo "No data found for key: ${key}"
  fi
done
