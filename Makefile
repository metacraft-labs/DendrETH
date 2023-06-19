yarn-check:
	yarn install --immutable --immutable-cache --silent || { \
		echo "Please run yarn install"; exit 1; \
	}

.PHONY: dendreth-relay-node
dendreth-relay-node:
	nix run '.#docker-image-yarn.copyToDockerDaemon'
	nix run '.?submodules=1#docker-image-all.copyToDockerDaemon'

	docker build -t metacraft/dendreth-relay-node -f Dockerfile.relay .

publish-dendreth-relay-node: dendreth-relay-node
	docker push metacraft/dendeth-relay-node

test-validator-accumulator: yarn-check
	cd beacon-light-client/solidity && \
	yarn hardhat test test/ValidatorAccumulator.test.ts

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

test-circom-circuits:
	./beacon-light-client/circom/test/run_snarkit2_tests.sh --force_recompile

test-plonky2-circuits:
	cd beacon-light-client/plonky2/circuits && \
	cargo test --release
