#!/usr/bin/env sh

source "${BASH_SOURCE%/*}/../../../scripts/utils/paths.sh"

set -euo pipefail

(
cd $ROOT/beacon-light-client/plonky2/circuits_executables

echo $(pwd)

cargo run --bin final_layer --release -- --proof-storage-type file  --folder-name /mnt/solunka-server-dendreth/sepolia --protocol $1 --redis redis://solunska-server:6379/
)
