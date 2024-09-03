#!/bin/sh

echo "Configuration:"
echo "    VALIDATORS_ACCUM_CONTRACT: $VALIDATORS_ACCUM_CONTRACT"
echo "    EXECUTION_NODE: $EXECUTION_NODE"
echo "    REDIS_AUTH: ${#REDIS_AUTH}"
echo "    REDIS_HOST: $REDIS_HOST"
echo "    REDIS_PORT: $REDIS_PORT"

cd /app &&
    nix develop \
        --accept-flake-config \
        --extra-experimental-features 'nix-command flakes' \
        --command sh -c '\
            cd beacon-light-client/plonky2/input_fetchers && \
            yarn ts balance_verification/deposits_accumulator/runnable/run_pubkey_commitment_mapper_scheduler.ts \
                --rebuild \
                --protocol diva \
                --contract-address "$VALIDATORS_ACCUM_CONTRACT" \
                --json-rpc "$EXECUTION_NODE" \
                --redis-auth "$REDIS_AUTH" \
                --redis-host "$REDIS_HOST" \
                --redis-port "$REDIS_PORT" '
