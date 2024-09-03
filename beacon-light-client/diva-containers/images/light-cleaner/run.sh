#!/bin/sh

echo "Configuration:"
echo "    REDIS_AUTH: ${#REDIS_AUTH}"
echo "    REDIS_HOST: $REDIS_HOST"
echo "    REDIS_PORT: $REDIS_PORT"

cd /app &&
    nix develop \
        --accept-flake-config \
        --extra-experimental-features 'nix-command flakes' \
        --command sh -c '\
            cd beacon-light-client/plonky2/input_fetchers && \
            yarn ts validators_commitment_mapper/runnable/light_cleaner.ts \
                --redis-auth "$REDIS_AUTH" \
                --redis-host "$REDIS_HOST" \
                --redis-port "$REDIS_PORT" '
