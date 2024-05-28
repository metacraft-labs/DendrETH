#!/usr/bin/env bash

# Define variables
REPO_URL="https://github.com/ethereum/bls12-381-tests"
REPO_DIR="bls12-381-tests"
OUTPUT_DIR="eth_tests"
VERIFY_DIR="$OUTPUT_DIR/verify"
SCRIPTS_DIR="scripts"
RUST_FILES_DIR="src" 

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

# Navigate back to the scripts directory
cd ..

# Navigate back to the root directory
cd ..

# Navigate to the rust files directory
cd "$RUST_FILES_DIR" || exit 

# Run the verify tests
run_verify_tests() {
    local test_name=$1
    local file_path=$2

    # Set the FILE_PATH environment variable
    export FILE_PATH="$file_path" 

    # Run the specified Rust test with the given file path
    RUST_MIN_STACK=16777216 cargo test "$test_name" --release -- --nocapture "$file_path"
}

# Loop through the extracted files in the 'verify' directory
for file in "$VERIFY_DIR"/*.yaml; do
  if [[ "$file" == *"wrong"* || "$file" == *"tampered"* ]]; then
    run_verify_tests "test_bls12_381_components_proofs_with_verify_eth_cases_should_fail" "$file"
  else
    run_verify_tests "test_bls12_381_components_proofs_with_verify_eth_cases" "$file"
  fi
done