CIRCUITS = aggregate_bitmask compress compute_domain compute_signing_root expand_message hash_to_field hash_tree_root hash_tree_root_beacon_header is_supermajority is_valid_merkle_branch light_client light_client_recursive
SUFFIXES = _cpp _build _zkey_0 _phase_2_ceremony _zkey _vkey _full
yarn-check:
	yarn install --immutable --immutable-cache --silent || { \
		echo "Please run yarn install"; exit 1; \
	}

.PHONY: build-relay-image
build-relay-image:
	nix run --builders '$(BUILDERS)' '.#docker-image-yarn.copyToDockerDaemon'
	nix run --builders '$(BUILDERS)' '.?submodules=1#docker-image-all.copyToDockerDaemon'

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

test-circom-circuits:
	cd beacon-light-client/circom && \
	./test/run_snarkit2_tests.sh

$(foreach s, $(SUFFIXES), $(addsuffix $s, $(CIRCUITS))) cosmos-verifier-parse-data cosmos-verifier-contract cosmos-light-client-contract cosmos-groth16-verifier beacon-light-client-wasm beacon-light-client-emmc-wasm: %: #yarn-check
	nix build --builders '$(BUILDERS)' -L  '.#packages.x86_64-linux.'$@


