#!/usr/bin/env bash

set -euo pipefail

source $(dirname ${BASH_SOURCE[0]})/parse_common_cmdline_opts.sh

# Reads SERVER_LIST from .env
if [ -z "$SERVER_LIST" ]; then
    echo "Failed to load SERVER_LIST environment variable from .env file."
    exit 1
else
    echo "SERVER_LIST loaded successfully."
fi

# Check if the FINAL_SERVER variable is set
if [ -z "$FINAL_SERVER" ]; then
    echo "Failed to load FINAL_SERVER environment variable from .env file."
    exit 1
else
    echo "FINAL_SERVER loaded successfully."
fi

# Loop through the server array and run commands in parallel without logs
for server in $SERVER_LIST
do
  echo "Starting on ${server} in background..."
  # Pass the first argument from run_everywhere.sh to run.sh
  ssh "$server" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/crates/circuit_executables/scripts/run_balance_verification.sh --proof-storage-dir $PROOF_STORAGE_DIR --redis-address $REDIS_ADDRESS --protocol $PROTOCOL" 2>&1 &
done

# Wait for all background jobs to finish
wait
echo "All jobs completed"

ssh "$FINAL_SERVER" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/crates/circuit_executables/scripts/run_final_layer.sh --proof-storage-dir $PROOF_STORAGE_DIR --redis-address $REDIS_ADDRESS --protocol $PROTOCOL" 2>&1

echo "Final layer completed"
