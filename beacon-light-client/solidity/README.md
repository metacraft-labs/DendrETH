This is solidity implementation of the Eth2 light client.
With the verification of the BLS12-381 signatures. Moved to a ZK circuit and verified on chain.

To run simulation of the light client updates using the aldready precomputed
proofs for updates run `yarn hardhat test test/BeaconLightClientReadyProofs.test.ts` in `src\solidity` directory.

To run simulation of the light client updates generating the proofs yourself run
`yarn hardhat test test/BeaconLightClient.test.ts` in `src\solidity` directory, requiring ~64GB ram to generate the proofs.

For this you need to have the `proof_efficient.circom` circuit compiled executing the `./build_proof.sh`, which requires 384GB of ram.
