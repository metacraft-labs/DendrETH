#!/usr/bin/env bash

set -euo pipefail

# ANSI color codes
GREEN='\033[0;32m'
NC='\033[0m'  # No Color

log() {
    local msg="$1"
    echo -e "\n$(date +'%Y-%m-%d %H:%M:%S') - $msg\n"
}

run_rust_command() {
    local bin_name="$1"
    local RUST_MIN_STACK="${2:-16777216}"  # Default value for RUST_MIN_STACK if not provided
    log "Starting ${GREEN}${bin_name}${NC} with RUST_MIN_STACK=$RUST_MIN_STACK"
    RUST_MIN_STACK=16777216 cargo run --bin "$bin_name" --release
    log "Finished ${GREEN}${bin_name}${NC}"
}

run_rust_command "calc_pairing_precomp_circuit_data_generation"
run_rust_command "final_exponentiate_circuit_data_generation"
run_rust_command "fp12_mul_circuit_data_generation"
run_rust_command "miller_loop_circuit_data_generation"
run_rust_command "bls12_381_circuit_data_generation"
