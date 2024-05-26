#!/usr/bin/env bash

set -euo pipefail

source $(dirname ${BASH_SOURCE[0]})/parse_common_cmdline_opts.sh

pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables
cargo run --bin final_layer --release -- --proof-storage-type file  --folder-name $PROOF_STORAGE_DIR --protocol $PROTOCOL --redis $REDIS_ADDRESS
command popd "$@" > /dev/null
