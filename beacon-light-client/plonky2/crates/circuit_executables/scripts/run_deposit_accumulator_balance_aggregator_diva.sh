#!/usr/bin/env bash

set -euo pipefail

source $(dirname ${BASH_SOURCE[0]})/parse_common_cmdline_opts.sh

command pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables "$@" > /dev/null

for i in $(seq 0 32); do
    cargo run --bin deposit_accumulator_balance_aggregator_diva --release -- --preserve-intermediary-proofs --level $i --stop-after 0 --protocol $PROTOCOL --proof-storage-type file --folder-name $PROOF_STORAGE_DIR --redis $REDIS_ADDRESS
done

command popd "$@" > /dev/null
