#!/usr/bin/env bash

# Define variables
REPO_URL="https://github.com/ethereum/bls12-381-tests"
REPO_DIR="bls12-381-tests"
OUTPUT_DIR="eth_tests"
VERIFY_DIR="$OUTPUT_DIR/bls/verify"
FROM_CIRCUIT_EXECUTABLES_TO="../../../.."
VENDOR="vendor"

command pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables "$@" > /dev/null

# Store the current directory
CIRCUIT_EXECUTABLES_DIR=$(pwd)

cd "$FROM_CIRCUIT_EXECUTABLES_TO/$VENDOR"

# Clone the repository
if [ ! -d "$REPO_DIR" ]; then
  git clone "$REPO_URL"
else
  echo "Repository already cloned."
fi

# Navigate to the repository directory
cd "$REPO_DIR" || exit

# Set up a Python virtual environment
if [ ! -d "venv" ]; then
  python -m venv venv
else
  echo "Virtual environment already set up."
fi

# Activate the virtual environment
# shellcheck source=/dev/null
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Run the test generator
python main.py --output-dir="$OUTPUT_DIR" --encoding=yaml

# Deactivate the virtual environment
deactivate

# Navigate to the circuit executables directory
cd "../.."
cd "$CIRCUIT_EXECUTABLES_DIR"

# Store all files in a variable
mapfile -t all_yaml_files_in_verify < <(ls *) 

# Run the verify tests
run_verify_tests() {
  local test_name=$1
  local file_path=$2

  # Set the FILE_PATH environment variable
  export FILE_PATH="$file_path" 

  # Run the specified Rust test with the given file path
  RUST_MIN_STACK=1116777216 cargo test "$test_name" --release -- --nocapture "$file_path"
}

# Loop through the extracted files in the 'verify' directory
for yaml_file in "${all_yaml_files_in_verify[@]}"; do
  run_verify_tests "test_bls12_381_components_proofs_with_verify_eth_cases" "$yaml_file"
done

command popd "$@" > /dev/null