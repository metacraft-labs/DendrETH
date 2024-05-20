#!/usr/bin/env sh

for i in $(seq $1 37); do
    cargo run --bin balance_verification --release -- --preserve-intermediary-proofs --level $i --proof-storage-type file --folder-name $WORK/proof_storage --stop-after 0 --protocol $2
done

cargo run --bin final_layer --release -- --proof-storage-type file --folder-name proofs --protocol $2
