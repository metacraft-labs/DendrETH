#!/usr/bin/env bash

# run the polling update task
yarn run pollUpdatesWorker > pollUpdatesWorker.log 2>&1 &

# run the proof generation task
yarn run proofGenerationWorker > proofGenerationWorker.log 2>&1 &

cd ../beacon-light-client/solidity

yarn hardhat run-update --initialslot 5248353 --slotsjump 64 &

yarn hardhat start-publishing --lightclient 0xFb3Bb7992A49703D4f3AEAA2FA95AA250aBE2936 --network goerli > goerli.log &

yarn hardhat start-publishing --lightclient 0x1a2FAA5f49385EebA349fd2616BAbf1Eb4367dcc --network optimisticGoerli > optimisticGoerli.log &

yarn hardhat start-publishing --lightclient 0xB94868ba0903883bD2dE3311Fc377f3c50D602eA --network baseGoerli > baseGoerli.log &

yarn hardhat start-publishing --lightclient 0xA3418F79c98A3E496A5E97610a97f82daE364619 --network arbitrumGoerli > arbitrumGoerli.log &

yarn hardhat start-publishing --lightclient 0xA3418F79c98A3E496A5E97610a97f82daE364619 --network sepolia > sepolia.log &

yarn hardhat start-publishing --lightclient 0xA3418F79c98A3E496A5E97610a97f82daE364619 --network mumbai > mumbai.log
