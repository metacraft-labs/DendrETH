require("dotenv").config();

// import "hardhat-gas-reporter";

import "@nomiclabs/hardhat-waffle";
import "@nomiclabs/hardhat-etherscan";
import "@nomiclabs/hardhat-ethers";

import "./tasks";

export default {
  solidity: {
    version: "0.8.9",
    settings: {
      optimizer: {
        enabled: true,
        runs: 1,
      },
    },
  },
  defaultNetwork: "hardhat",
  networks: {
    local: {
      url: "http://127.0.0.1:8545/",
      accounts: [
        process.env.LOCAL_NETWORK_PRIVATE_KEY,
      ],
    },
    hardhat: {
      blockGasLimit: 30000000,
      allowUnlimitedContractSize: true,
    },
    ropsten: {
      url: `https://ropsten.infura.io/v3/${process.env.ROPSTEN_NETWORK_INFURA_API_KEY}`,
      accounts: [
        process.env.ROPSTEN_NETWORK_PRIVATE_KEY,
      ],
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,
  },
};
