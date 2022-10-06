This folder contains a complete Solidity implementation of a beacon chain light client. The verification of BLS12-381 signatures is based on a zero-knowledge circuit developed in the [circom](../circom) folder.
Please set up the required environment variables by renaming ``.env.example`` to ``.env`` and populating the necessary fields.

## Tests

You can run a syncing simulation based on pre-computed light client updates and proofs through the following command:

```bash
yarn hardhat test test/BeaconLightClientReadyProofs.test.ts
```

To generate the proofs yourself, run:

```bash
yarn hardhat test test/BeaconLightClient.test.ts
```

This would require roughly 64GB of RAM. Prior to this, you need to compile the `proof_efficient.circom` circuit (potentially on a different machine) through the [`./build_proof.sh`](../circom/scripts/proof_efficient/build_proof.sh) script which will require roughly 340GB of RAM.
