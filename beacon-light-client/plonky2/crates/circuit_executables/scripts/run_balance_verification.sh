#!/usr/bin/env bash

set -euo pipefail

source $(dirname ${BASH_SOURCE[0]})/parse_common_cmdline_opts.sh

pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables

for i in $(seq 0 37); do
    cargo run --bin balance_verification --release -- --preserve-intermediary-proofs --level $i --stop-after 0 --protocol $PROTOCOL --proof-storage-type file --folder-name $PROOF_STORAGE_DIR --redis $REDIS_ADDRESS
done

command popd "$@" > /dev/null
