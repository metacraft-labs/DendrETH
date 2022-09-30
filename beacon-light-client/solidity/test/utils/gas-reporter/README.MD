# GAS REPORTER

### HOW TO USE

1. Simply execute `npm run gas` or `yarn run gas` in the `DendrETH/beacon-light-client/solidity` directory
2. Then check out the generated gas report (a `.txt` file) in `DendrETH/beacon-light-client/solidity/test/utils/gas-reporter/output`

### DEPENDENCIES

The gas reporter is currently dependent on the following local functions/files:

1. `getConstructorArgs` in `DendrETH/beacon-light-client/solidity/tasks/utils/index.ts` - parameters needed to deploy the BeaconLightClient contract in the gas cost measurement environment
2. `DendrETH/beacon-light-client/solidity/contracts/` - get and compile all beacon light client contracts
3. `DendrETH/vendor/eth2-light-client-updates/mainnet/updates/` - real updates which are used in the gas cost measurement environment
4. `DendrETH/beacon-light-client/solidity/test/` - some common functions are used from the `test` directory such as `getFilesInDir`, `getSolidityProof`, etc.

**NB: All `.sol` files should be formatted when running the `npm run gas`(`yarn run gas`) command. Otherwise problems may occur when measuring gas cost or compiling contracts (e.g. not formatting the `DendrETH/beacon-light-client/solidity/bridge/src/utils/Verifier.sol` will cause such problems)**
