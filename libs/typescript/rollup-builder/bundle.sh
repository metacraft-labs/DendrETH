#!/usr/bin/env bash

SCRIPTDIR=$(dirname "$(readlink -f "$0")")
echo "dirpath: $SCRIPTDIR"
(
  command pushd "$SCRIPTDIR" > /dev/null
  yarn rollup --silent --config ./rollup.config.mjs
  command popd > /dev/null
)

