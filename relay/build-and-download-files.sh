#!/usr/bin/env bash

cd ../

git submodule update --init --recursive
yarn install --production

# Build rapidsnark
cd vendor/rapidsnark
npm install
git submodule init
git submodule update
npx task createFieldSources
npx task buildProver

cd ../../

BUILD_DIR="build"

if [ ! -d "$BUILD_DIR" ]; then
  mkdir "$BUILD_DIR"
fi

cd "$BUILD_DIR"

if [! -d light_client.zkey ]; then
  curl http://dendreth.metacraft-labs.com/capella_74.zkey > light_client.zkey
fi

cd ../

# Compile smart contracts
cd beacon-light-client/solidity
yarn hardhat compile
