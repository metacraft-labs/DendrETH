#!/bin/sh

echo "Configuration:"
echo "    REDIS_AUTH: ${#REDIS_AUTH}"
echo "    REDIS_HOST: $REDIS_HOST"
echo "    REDIS_PORT: $REDIS_PORT"

export REDIS="redis://${REDIS_AUTH}@${REDIS_HOST}:${REDIS_PORT}"

cd /app &&
    nix develop \
        --accept-flake-config \
        --extra-experimental-features 'nix-command flakes' \
        --command sh -c '\
            cd beacon-light-client/plonky2/crates/circuit_executables && \
            cargo run \
                --bin pubkey_commitment_mapper \
                --release -- \
                    --proof-storage-type file \
                    --folder-name /app/proofs \
                    --redis "$REDIS" \
                    --protocol diva '
