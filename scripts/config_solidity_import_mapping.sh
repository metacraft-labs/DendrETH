#!/usr/bin/env bash

REMAPPING_FILE_NAME="remappings.txt"
CONTRACTS_DIR="$GIT_ROOT/beacon-light-client/solidity"

OPENZEPPELIN_LIB="$(find "$GIT_ROOT"/.yarn/unplugged -maxdepth 1 -type d -name '@openzeppelin-*')/node_modules/@openzeppelin/"
OPENZEPPELIN_REMAP="@openzeppelin/=$OPENZEPPELIN_LIB"

echo "$OPENZEPPELIN_REMAP" >"$CONTRACTS_DIR/$REMAPPING_FILE_NAME"
