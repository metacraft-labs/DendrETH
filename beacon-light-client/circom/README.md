This folder contains the building blocks of the DendrETH light client syncing circuit
which are assembled in the final `proof_efficient` circuit.

## Tests

You can use the following command to run all tests:

```
yarn hardhat test
```

## Building

In the scripts folder, you'll find the circuits with main components. You can build them with the provided shell scripts and use the provided JavaScript files for producing example inputs. Please note that compiling some of the larger circuits is expected to take multiple hours and may require a computer with hundreds of GB of RAM.

As an example, here are our build times from a 32-core, 384G RAM machine with a 1TB NVMe hard drive, configured with 500GB of swap space. Constraints refer to non-linear constraints.

|                                      | proof_efficient |
| ------------------------------------ | --------------- |
| Constraints                          | 88945803        |
| Circuit compilation                  | 6h              |
| Witness generation C++ compilation   | 1h              |
| Witness generation                   | 3m              |
| Trusted setup phase 2 key generation | 26h             |
| Trusted setup phase 2 contribution   | N/a             |
| Proving key size                     | 49G             |
| Proving key verification             | N/a             |
| Proving time (rapidsnark)            | 4m              |
| Proof verification time              | 1s              |

## Diagrams

These crude diagrams may help you understand the interactions between all components of the system better:

![](light_client.drawio.png)
![](zero_knowledge_diagram.drawio.png)
