#!/usr/bin/env bash

source "${GIT_ROOT}/.env"

# Check if Redis is running
if pgrep prometheus >/dev/null; then
  echo "Prometheus is already running."
else
  echo "Prometheus is not running. Starting a new one"
  (
    cd "${GIT_ROOT}/relay" || exit 1
    prometheus --config.file=prometheus.yml
  )
fi
