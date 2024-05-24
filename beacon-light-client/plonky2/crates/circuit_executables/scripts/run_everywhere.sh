#!/usr/bin/env sh

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
  ssh "$server" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/circuits_executables/run.sh 0 $1" 2>&1 &
done

# Wait for all background jobs to finish
wait
echo "All jobs completed"

ssh "$FINAL_SERVER" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/circuits_executables/run_final_layer.sh $1" 2>&1

echo "Final layer completed"
