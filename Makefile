yarn-check:
	yarn install --immutable --immutable-cache --silent || { \
		echo "Please run yarn install"; exit 1; \
	}

evm-simulation: yarn-check
	cd beacon-light-client/solidity && \
	yarn hardhat test test/BeaconLightClientReadyProofs.test.ts
