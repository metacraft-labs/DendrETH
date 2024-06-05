#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(dirname "${BASH_SOURCE[0]}")"

# shellcheck source=./get-host-system
. "$SCRIPT_DIR/get-host-system"

system="$(get_host_system)"

set -x

# if [[ -n "${GITHUB_ENV:-}" ]]; then
#   git config --global url."git@github.com:".insteadOf https://github.com/
#   git config --global url."git://".insteadOf https://
# fi

nix build --json --print-build-logs ".#devShells.$system.default"
