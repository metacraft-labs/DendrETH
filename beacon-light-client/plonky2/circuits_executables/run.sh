#!/usr/bin/env sh

for i in $(seq $1 37); do
    cargo run --bin balance_verification --release -- --preserve-intermediary-proofs --level $i --proof-storage-type file --folder-name proofs
done
