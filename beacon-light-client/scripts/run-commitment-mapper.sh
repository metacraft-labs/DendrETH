#!/usr/bin/env bash

set -euo pipefail

function join_by { set +x; local IFS="$1"; shift; echo "$*"; set -x; }

SSHFS_OPTS_ARRAY=(
  reconnect
  ServerAliveInterval=15
  ServerAliveCountMax=3
  default_permissions
  idmap=user
  umask=000
  cache=yes
  kernel_cache
  compression=no
  Ciphers=aes128-gcm@openssh.com
)

set -x

SERVER_IP="${SERVER_IP}"

SSHFS_OPTS="$(join_by , "${SSHFS_OPTS_ARRAY[@]}")"

SSHFS_USER="${SSHFS_USER:-dendreth}"
SSHFS_HOST="${SSHFS_HOST:-$SERVER_IP}"

SSHFS_REMOTE_DIR="${SSHFS_REMOTE_DIR:-/storage/dendreth-proof-storage/mainnet}"
SSHFS_MOUNTPOINT="${SSHFS_MOUNTPOINT:-$HOME/dendreth_proof_storage}"
SSHFS_CONNECTION_STRING="${SSHFS_CONNECTION_STRING:-$SSHFS_USER@$SSHFS_HOST:$SSHFS_REMOTE_DIR}"

REDIS_HOST="${REDIS_HOST:-$SERVER_IP}"
REDIS_PORT="${REDIS_PORT:-6000}"
REDIS_AUTH="${REDIS_AUTH:+$REDIS_AUTH@}"
REDIS_CONNECTION_STRING="${REDIS_CONNECTION_STRING:-redis://${REDIS_AUTH}${REDIS_HOST}:${REDIS_PORT}}"

BRANCH="${BRANCH:-diva-deployment}"
FLAKE_REF_BASE="${FLAKE_REF_BASE:-github:metacraft-labs/DendrETH}"
FLAKE_REF="${FLAKE_REF:-$FLAKE_REF_BASE/$BRANCH}"
FLAKE_ATTR_PATH="${FLAKE_ATTR_PATH:-circuit-executables.commitment-mapper.levels.0.pkg}"
FLAKE="${FLAKE:-${FLAKE_REF}#${FLAKE_ATTR_PATH}}"

mkdir -p "$SSHFS_MOUNTPOINT"
mountpoint -q "$SSHFS_MOUNTPOINT" && {
  fusermount -uz "$SSHFS_MOUNTPOINT" \
  || {
    echo "Failed to unmount $SSHFS_MOUNTPOINT"
    echo "Processes still using $SSHFS_MOUNTPOINT:"
    lsof +f -- "$SSHFS_MOUNTPOINT" 2>/dev/null \
    | awk 'NR>1 {print $2}' \
    | xargs -r ps
    exit 1
  }
}

sshfs \
  -o "${SSHFS_OPTS}" \
  "$SSHFS_CONNECTION_STRING" \
  "$SSHFS_MOUNTPOINT"

RUST_BACKTRACE=1 nix run --refresh \
  "$FLAKE" \
  -- \
  --redis "$REDIS_CONNECTION_STRING" \
  --proof-storage-type file \
  --folder-name "$SSHFS_MOUNTPOINT"
