## Introduction

The relay directory contains another directory with 2 scripts - one is for Prater and the other for Mainnet. Both fetch requests and receive respones, which are in JSON format and can be used for simulations and testing.

## About the scripts

The scripts are fully functional and the only requirement is that the local machine, on which they are run, has being synchronized with the Beacon chain(either mainnet or testnet). The only thing that should be POTENTIALLY changed in the scripts are the ports, depending on which port the beacon node has been set up to and the path, from which a snapshot is being read to start a synchronization and to which the received data, consisting of JSON files, will be written (could be read from and written to the same directory like [here](https://github.com/metacraft-labs/eth2-light-client-updates).  