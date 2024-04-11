#!/usr/bin/env bash

SNARKJS_LIB_NAME="snarkjs-npm-0.4.10*"
SNARKJS_LIB="$(find "$ROOT/.yarn/unplugged" -maxdepth 1 -type d -name "$SNARKJS_LIB_NAME")/node_modules/snarkjs"
SNARKJS_CLI="$SNARKJS_LIB/cli.js"
