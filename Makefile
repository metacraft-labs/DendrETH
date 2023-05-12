yarn-check:
	yarn install --immutable --immutable-cache --silent || { \
		echo "Please run yarn install"; exit 1; \
	}

.PHONY: build-relay-image
build-relay-image:
	nix run '.#docker-image-yarn.copyToDockerDaemon'
	nix run '.?submodules=1#docker-image-all.copyToDockerDaemon'

	docker build -t relayimage -f Dockerfile.relay .

evm-simulation: yarn-check
	cd beacon-light-client/solidity && \
	yarn hardhat test test/BeaconLightClientReadyProofs.test.ts

one-shot-syncing-simulation: yarn-check
	cd beacon-light-client/circom && \
	yarn hardhat run scripts/light_client_recursive/verify_updates.ts

test-groth16-verifier:
	nim c -r tests/nim-groth16-verifier/verifier_test.nim

test-solidity-beacon-light-client-verifier:
	cd beacon-light-client/solidity && \
	yarn hardhat test test/BeaconLightClientReadyProofs.test.ts
