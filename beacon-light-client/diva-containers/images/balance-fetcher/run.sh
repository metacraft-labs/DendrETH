#!/bin/sh

echo "Configuration:"
echo "    EXECUTION_NODE: $EXECUTION_NODE"
echo "    BEACON_NODE: $BEACON_NODE"
echo "    VALIDATORS_ACCUM_CONTRACT: $VALIDATORS_ACCUM_CONTRACT"
echo "    SNAPSHOT_CONTRACT: $SNAPSHOTS_CONTRACT"
echo "    REDIS_AUTH: ${#REDIS_AUTH}"
echo "    REDIS_HOST: $REDIS_HOST"
echo "    REDIS_PORT: $REDIS_PORT"

cd /app &&
    nix develop \
        --accept-flake-config \
        --extra-experimental-features 'nix-command flakes' \
        --command sh -c '\
            cd beacon-light-client/plonky2/input_fetchers && \
            yarn ts balance_verification/deposits_accumulator/runnable/diva_balance_aggregator_scheduler.ts \
                --json-rpc "$EXECUTION_NODE" \
                --beacon-node "$BEACON_NODE" \
                --address  "$VALIDATORS_ACCUM_CONTRACT" \
                --snapshot-contract-address "$SNAPSHOT_CONTRACT" \
                --redis-auth "$REDIS_AUTH" \
                --redis-host "$REDIS_HOST" \
                --redis-port "$REDIS_PORT" \
                --protocol diva'
