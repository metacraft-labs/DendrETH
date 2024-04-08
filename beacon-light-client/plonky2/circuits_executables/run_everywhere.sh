#!/usr/bin/env sh

# Define an array of your server names
servers=("gpu-server-001" "gpu-server-002" "gpu-server-003")

# Loop through the server array and run commands in parallel without logs
for server in "${servers[@]}"
do
  echo "Starting on ${server} in background..."
  # Pass the first argument from run_everywhere.sh to run.sh
  ssh "$server" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/circuits_executables/run.sh 0 $1" 2>&1 &
done

# Wait for all background jobs to finish
wait
echo "All jobs completed"

ssh "gpu-server-002" "cd ~/DendrETH && nix develop -c ./beacon-light-client/plonky2/circuits_executables/run_final_layer.sh $1" 2>&1

echo "Final layer completed"
