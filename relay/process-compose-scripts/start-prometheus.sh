#!/usr/bin/env bash

source "${BASH_SOURCE%/*}/../../scripts/utils/paths.sh"
source "${ROOT}/.env"

# Check if Redis is running
if pgrep prometheus >/dev/null; then
  echo "Prometheus is already running."
else
  echo "Prometheus is not running. Starting a new one"
  (
    cd "${ROOT}/relay" || exit 1
    prometheus --config.file=prometheus.yml
  )
fi
