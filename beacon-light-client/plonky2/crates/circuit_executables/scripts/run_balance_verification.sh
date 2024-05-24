#!/usr/bin/env sh

set -euo pipefail

(
pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables

echo $(pwd)

for i in $(seq $1 37); do
    cargo run --bin balance_verification --release -- --preserve-intermediary-proofs --level $i --proof-storage-type file --folder-name proofs --stop-after 0 --protocol $2
done

 cargo run --bin final_layer --release -- --proof-storage-type file  --folder-name proofs --protocol $2
 # cargo run --bin final_layer --release -- --proof-storage-type file  --folder-name /mnt/solunka-server-dendreth/sepolia --protocol $2 --redis redis://solunska-server:6379/
 popd
)
