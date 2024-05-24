#!/usr/bin/env sh

set -euo pipefail

(
pushd $GIT_ROOT/beacon-light-client/plonky2/crates/circuit_executables

echo $(pwd)

cargo run --bin commitment_mapper --release -- --proof-storage-type file --folder-name proofs
 popd
)
