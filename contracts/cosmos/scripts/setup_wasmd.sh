#!/bin/bash
set -o errexit -o nounset -o pipefail

PASSWORD=${PASSWORD:-1234567890}
STAKE=${STAKE_TOKEN:-ustake}
FEE=${FEE_TOKEN:-ucosm}
CHAIN_ID=${CHAIN_ID:-testing}
MONIKER=${MONIKER:-node001}
KEYRING="--keyring-backend test --keyring-dir $HOME/.wasmd_keys"

# Clean-up old data
rm -rf "$HOME"/.wasmd/

wasmd init --chain-id "$CHAIN_ID" "$MONIKER"
# staking/governance token is hardcoded in config, change this
## OSX requires: -i.
sed -i. "s/\"stake\"/\"$STAKE\"/" "$HOME"/.wasmd/config/genesis.json
if ! wasmd keys show validator $KEYRING; then
  (
    echo "$PASSWORD"
    echo "$PASSWORD"
  ) | wasmd keys add validator $KEYRING
fi
# hardcode the validator account for this instance
echo "$PASSWORD" | wasmd add-genesis-account $(wasmd keys show -a validator $KEYRING) "1000000000$STAKE,1000000000$FEE"
# (optionally) add a few more genesis accounts
for addr in "$@"; do
  echo "$addr"
  wasmd add-genesis-account "$addr" "1000000000$STAKE,1000000000$FEE"
done
# submit a genesis validator tx
## Workraround for https://github.com/cosmos/cosmos-sdk/issues/8251
(
  echo "$PASSWORD"
  echo "$PASSWORD"
  echo "$PASSWORD"
) | wasmd gentx validator "250000000$STAKE" --chain-id="$CHAIN_ID" --amount="250000000$STAKE" $KEYRING
## should be:
# (echo "$PASSWORD"; echo "$PASSWORD"; echo "$PASSWORD") | wasmd gentx validator "250000000$STAKE" --chain-id="$CHAIN_ID"
wasmd collect-gentxs
