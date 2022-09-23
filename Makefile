yarn-check:
	yarn install --immutable --immutable-cache --silent || { echo "Please run yarn install"; }

evm-simulation: yarn-check
	cd beacon-light-client/solidity && \
	yarn hardhat test test/BeaconLightClientReadyProofs.test.ts
