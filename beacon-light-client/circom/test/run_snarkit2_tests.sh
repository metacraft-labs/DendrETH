#!/usr/bin/env bash

# Check if --force_recompile argument was passed
if [ "$1" == "--force_recompile" ]; then
  extra_args="--force_recompile"
else
  extra_args=""
fi

# generate the ssz_num test cases
(
  cd "$GIT_ROOT/beacon-light-client/circom/test"
  yarn tsx ./gen_ssz_num_positive_tests.ts
)

# Get the list of all directories one level deep inside the "test" folder
folders=$(find ${GIT_ROOT}/beacon-light-client/circom/test/* -maxdepth 0 -type d)

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

# remove the generated ssz_num_uint test cases
find . -type d -name 'ssz_num_uint*' -exec rm -rf {} +

echo "All tests completed."
