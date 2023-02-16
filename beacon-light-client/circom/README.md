This folder contains the building blocks of the DendrETH light client syncing circuit
which are assembled in the final `light_client` circuit.

## Tests

There two kind of tests to run the `circom_tester` tests you can execute:

```
yarn hardhat test
```

To run the `snarkit2` tests you can execute:

```
yarn snarkit2 check .test/${folder_name}/ --witness_type bin --backend native
```

## Building

In the scripts folder, you'll find the circuits with main components. You can build them with the provided shell scripts and use the provided JavaScript files for producing example inputs. Please note that compiling some of the larger circuits is expected to take multiple hours and may require a computer with hundreds of GB of RAM.

As an example, here are our build times from a 32-core, 384G RAM machine with a 1TB NVMe hard drive, configured with 500GB of swap space.

|                                      | light_client |
| ------------------------------------ | ------------ |
| Constraints                          | 89436966     |
| Circuit compilation                  | 13.7h        |
| Witness generation C++ compilation   | 43m          |
| Witness generation                   | 3m           |
| Trusted setup phase 2 key generation | 26h          |
| Trusted setup phase 2 contribution   | N/a          |
| Proving key size                     | 49G          |
| Proving key verification             | N/a          |
| Proving time (rapidsnark)            | 4m           |

To build the circuits you need Powers of Tau file with `2^28` constraints
You can download it from this repository <https://github.com/iden3/snarkjs#7-prepare-phase-2>
And place it in `circuits/build` folder

Than you can just enter the `scripts/light_client` to build the light client circuit.

```
./build_proof.sh
```

It will generate the witness generator program and the zkey file of the circuit.

To run the `relayer` which will poll a beacon node for a finality update every minute. You need to run

```
yarn ts-node relayer.ts
```

And the two workers

```
yarn ts-node get-update-worket.ts
yarn ts-node proof-generator-worker.ts
```

For this to work you need to have at least one previous update in `relayer_updates` folder and have the the `state.json` filled.

The relayer will dispatch a task for downloading updates every minute.
After this the proof worker can start generating proofs.
The relayer will also listen for when proofs are ready and publish them on chain in the correct order.

<!-- ## Diagrams

These crude diagrams may help you understand the interactions between all components of the system better:

![](light_client.drawio.png)
![](zero_knowledge_diagram.drawio.png) -->
