#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Usage: $0 <fuzz_test_name> [additional_args...]"
    exit 1
fi

FUZZ_TEST_NAME=$1
FUZZ_DIR=$(pwd)
ARTIFACTS_DIR="$FUZZ_DIR/artifacts/$FUZZ_TEST_NAME"
CORPUS_DIR="$FUZZ_DIR/corpus/$FUZZ_TEST_NAME"
ADDITIONAL_ARGS=("${@:2}")

# Create directories with sudo if necessary
mkdir -p "$ARTIFACTS_DIR"
mkdir -p "$CORPUS_DIR"

# Run the fuzzing command with additional arguments
RUSTFLAGS="--cfg fuzzing -Clink-dead-code -Cdebug-assertions -C codegen-units=1" \
    cargo run \
    --manifest-path "$FUZZ_DIR/Cargo.toml" \
    --target aarch64-apple-darwin \
    --release \
    --bin "$FUZZ_TEST_NAME" \
    -- -artifact_prefix="$FUZZ_DIR/artifacts/$FUZZ_TEST_NAME/" "$FUZZ_DIR/corpus/$FUZZ_TEST_NAME" "${ADDITIONAL_ARGS[@]}"
