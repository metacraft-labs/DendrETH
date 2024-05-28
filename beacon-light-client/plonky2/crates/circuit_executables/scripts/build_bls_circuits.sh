#!/usr/bin/env sh
#
set -euo pipefail

export RUST_MIN_STACK=16777216

command pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables "$@" > /dev/null

cargo run --bin fp12_mul_circuit_data_generation --release
cargo run --bin miller_loop_circuit_data_generation --release
cargo run --bin final_exponentiate_circuit_data_generation --release
cargo run --bin calc_pairing_precomp_circuit_data_generation --release
cargo run --bin bls12_381_circuit_data_generation --release

command popd "$@" > /dev/null
