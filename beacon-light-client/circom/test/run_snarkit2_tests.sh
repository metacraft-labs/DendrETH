#!/usr/bin/env bash

# Get the list of all directories one level deep inside the "test" folder
folders=$(find test/* -maxdepth 0 -type d)

# Loop through the directories and run tests for each folder
for folder in $folders; do
  echo "Running tests for: $folder"
  yarn snarkit2 test "$folder" --witness_type bin
  echo "----------------------------------------"
done

echo "All tests completed."
