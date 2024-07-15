#!/usr/bin/env bash

# This is a version of
# run_deposit_accumulator_balance_aggregator_diva.sh that uses AWS to
# store proofs and environment variables instead of command line
# arguments to pass secrets.
#
# Expected variables:
#
# ROOT (optional): the top DendrETH directory, defaults to /app
# AWS_ACCESS_KEY_ID
# AWS_BUCKET
# AWS_REGION
# AWS_SECRET_ACCESS_KEY
# REDIS_AUTH: e.g. default:PASSWORD
# REDIS_HOST
# REDIS_PORT

if [ -z "$ROOT" ] ; then
    ROOT=/app
fi

REDIS="redis://${REDIS_AUTH}@${REDIS_HOST}:${REDIS_PORT}"

set -o pipefail

(
    cd "$ROOT"/beacon-light-client/plonky2/crates/circuit_executables || exit 1

    for i in $(seq 0 32); do
        cargo run \
              --bin deposit_accumulator_balance_aggregator_diva \
              --release \
              -- \
              --preserve-intermediary-proofs \
              --level "$i" \
              --stop-after 0 \
              --protocol diva \
              --proof-storage-type aws \
              --folder-name /home/d/proofs \
              --redis "$REDIS" \
              --aws-bucket-name "$AWS_BUCKET" \
              --aws-region "$AWS_REGION" \
              --aws-endpoint-url ""
    done
)
