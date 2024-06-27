#!/usr/bin/env bash

# Load environment variables
source "${GIT_ROOT}/.env"

# Step 1: Start and time the ProverServer
./time_proverserver_start.sh

# Step 2: Run the TypeScript proof generation script
yarn ts ./path/to/proof_generation_script.ts
