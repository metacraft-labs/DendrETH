## Introduction

The DendrETH project aims to develop a beacon chain light client implementation
in the form of a smart contract for multiple targeted blockchains. See our
[roadmap](./docs/ROADMAP.md) for more details.

## Contributing

### Build environment

This project offers a fully-reproducible build environment based on the Nix
package manager. Due to the large number of dependencies required to target
and test the plethora of supported blockchains, all contributors are advised
to install Nix in order to take advantage of the provided automation. See our
[Getting started with Nix](docs/NIX.md) tutorial for more details.

### Running the test suite

To execute all tests, run `yarn test` at the root of this repo.
