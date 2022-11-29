#!/bin/bash
set -o errexit -o nounset -o pipefail

KEYRING="--keyring-backend test --keyring-dir $HOME/.wasmd_keys"

BASE_ACCOUNT=$(wasmd keys show validator -a $KEYRING)
wasmd q account "$BASE_ACCOUNT" -o json | jq

if wasmd keys show fred -a $KEYRING; then (echo -e "y \n") | wasmd keys delete fred $KEYRING; fi
echo "## Add new account"
(echo -e 'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone \n';) | wasmd keys add fred -i $KEYRING

echo "## Check balance"
NEW_ACCOUNT=$(wasmd keys show fred -a $KEYRING)
wasmd q bank balances "$NEW_ACCOUNT" -o json || true

echo "## Transfer tokens"
wasmd tx bank send validator "$NEW_ACCOUNT" 750000000ustake --gas 1000000 -y --chain-id=testing --node=http://localhost:26657 -b block -o json $KEYRING | jq

echo "## Check balance again"
wasmd q bank balances "$NEW_ACCOUNT" -o json | jq
