#!/usr/bin/env bash

DENDRETH_DIR=$(git rev-parse --show-toplevel)

# Get the list of all directories one level deep inside the "test" folder
folders=$(find ${DENDRETH_DIR}/beacon-light-client/circom/test/* -maxdepth 0 -type d)

# Check if --force_recompile argument was passed
if [ "$1" == "--force_recompile" ]; then
  extra_args="--force_recompile"
else
  extra_args=""
fi

# Loop through the directories and run tests for each folder
for folder in $folders; do
  echo "Running tests for: $folder"
  yarn snarkit2 test "$folder" --witness_type bin $extra_args

  # Check if the command was successful
  if [ $? -ne 0 ]; then
    echo "Test failed for: $folder"
    exit 1
  fi

  echo "----------------------------------------"
done

echo "All tests completed."
