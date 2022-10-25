yarn-check:
	yarn install --immutable --immutable-cache --silent || { \
		echo "Please run yarn install"; exit 1; \
	}

evm-simulation: yarn-check
	cd beacon-light-client/solidity && \
	yarn hardhat test test/BeaconLightClientReadyProofs.test.ts

one-shot-syncing-simulation: yarn-check
	cd beacon-light-client/circom && \
	yarn hardhat run scripts/light_client_recursive/verify_updates.ts
