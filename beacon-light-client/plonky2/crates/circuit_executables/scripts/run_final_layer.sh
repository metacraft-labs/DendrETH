#!/usr/bin/env sh

set -euo pipefail

(
pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables

echo $(pwd)

cargo run --bin final_layer --release -- --proof-storage-type file  --folder-name /mnt/solunka-server-dendreth/sepolia --protocol $1 --redis redis://solunska-server:6379/
popd
)
