#!/usr/bin/env bash

source "${GIT_ROOT}/.env"

# Check if Redis is running
if pgrep redis-server >/dev/null; then
  echo "Redis is already running."
else
  echo "Redis is not running. Starting a new one"

  # Check for redis environment variables
  if [[ -z "${REDIS_HOST}" ]] && [[ -z "${REDIS_PORT}" ]]; then
    echo "REDIS_HOST and REDIS_PORT environment variables are not set. Using default values."
    REDIS_HOST="localhost"
    REDIS_PORT="6379"
  else
    echo "Using Redis settings from environment variables"
  fi

  if [[ "${REDIS_HOST}" == "localhost" ]] && [[ "${REDIS_PORT}" == "6379" ]]; then
    echo "Starting local Redis server..."
    mkdir "${GIT_ROOT}/redis-server"
  (
    cd "${GIT_ROOT}/redis-server" || exit 1
    redis-server --appendonly yes

  )
    echo "Local Redis server started"
  else
    echo "Using remote Redis server at ${REDIS_HOST}:${REDIS_PORT}"
  fi
fi
