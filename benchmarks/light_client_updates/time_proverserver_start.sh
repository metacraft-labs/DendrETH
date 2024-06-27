#!/usr/bin/env bash

source "${GIT_ROOT}/.env"

# Define the log file for timing the server start
server_log_file="proverserver_start_time.log"

# Start the ProverServer
echo "Starting ProverServer..."
start_time=$(date +%s)
./start-proverserver.sh

# Wait for the ProverServer to be ready
./check-proverserver.sh

# Calculate the duration it took to start the server
end_time=$(date +%s)
start_duration=$((end_time - start_time))
echo "ProverServer started in ${start_duration} seconds." | tee -a "${server_log_file}"

echo "ProverServer is up and running."
