#!/bin/sh

cd /app &&                                                                       \
    nix develop                                                                  \
        --accept-flake-config                                                    \
        --extra-experimental-features 'nix-command flakes'                       \
        --command sh -c '                                                        \
            cd beacon-light-client/plonky2/crates/circuit_executables/scripts && \
            ./aggregate.sh                                                       \
        '
