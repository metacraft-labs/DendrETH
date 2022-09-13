To run simulation of the light client updates using the aldready precomputed proofs for updates run the `BeaconLightClientReadyProofs.test.ts`

To run simulation of the light client updates generating the proofs yourself run `BeaconLightClient.test.ts`. For this you need to have the `proof_efficient.circom` circuit compiled, which requires 384GB of ram. And 64GB ram to generate the proofs.
