To run simulation of the light client updates using the aldready precomputed proofs for updates run the `npx hardhat test test/BeaconLightClientReadyProofs.test.ts`

To run simulation of the light client updates generating the proofs yourself run `npx hardhat test test/BeaconLightClient.test.ts`, requiring ~64GB ram to generate the proofs.

For this you need to have the `proof_efficient.circom` circuit compiled executing the `./build_proof.sh`, which requires 384GB of ram.
