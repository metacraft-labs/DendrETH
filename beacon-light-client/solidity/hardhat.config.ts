require('dotenv').config();

import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-ethers';

import './tasks';

const optionalConf = {
  USER_PRIVATE_KEY: process.env.USER_PRIVATE_KEY,
};

const mandatoryConf = {
  LOCAL_HARDHAT_PRIVATE_KEY:
    process.env.LOCAL_HARDHAT_PRIVATE_KEY ||
    '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
  INFURA_API_KEY: process.env.INFURA_API_KEY,
  ETHERSCAN_API_KEY: process.env.ETHERSCAN_API_KEY,
  NODE_ID: process.env.NODE_ID || '0',
};

for (const envVar of Object.keys(mandatoryConf)) {
  if (!mandatoryConf[envVar]) {
    console.warn(`$${envVar} environment variable is not set`);
    process.exit(0);
  }
}

if (!/^0x[0-9a-fA-F]{64}$/.test(optionalConf.USER_PRIVATE_KEY ?? '')) {
  console.warn(
    'Setting $USER_PRIVATE_KEY to $LOCAL_HARDHAT_PRIVATE_KEY as fallback',
  );
  optionalConf.USER_PRIVATE_KEY = mandatoryConf.LOCAL_HARDHAT_PRIVATE_KEY;
}

export default {
  solidity: {
    version: '0.8.9',
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  defaultNetwork: 'hardhat',
  networks: {
    local: {
      url: 'http://127.0.0.1:8545/',
      accounts: [mandatoryConf.LOCAL_HARDHAT_PRIVATE_KEY],
    },
    hardhat: {
      blockGasLimit: 30000000,
      allowUnlimitedContractSize: true,
    },
    ropsten: {
      url: `https://ropsten.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    mainnet: {
      url: `https://goerli.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    "fantom-test": {
      url: "https://rpc.testnet.fantom.network",
      accounts: [optionalConf.USER_PRIVATE_KEY],
      chainId: 4002,
      live: false,
      saveDeployments: true,
      gasMultiplier: 2,
    },
    "avalanche-mainnet": {
      url: `https://avalanche-mainnet.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    "avalanche-fuji": {
      url: `https://avalanche-fuji.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    "celo-mainnet": {
      url: `https://celo-mainnet.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    "celo-alfajores": {
      url: `https://celo-alfajores.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    //WIP, getting error "could not detect network"
    hedera: {
      url: `https://${mandatoryConf.NODE_ID}.testnet.hedera.com`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    //WIP, faucet claims i'm a robot, and won't give me BNB
    "binance-smart-contract": {
      url: `https://data-seed-prebsc-1-s1.binance.org:8545`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    //WIP, no faucet with which to gain TFUEL, theta RPC returns an error: -32000: Failed to get the account (the address has no Theta nor TFuel)
    "theta": {
      url: `https://eth-rpc-api-testnet.thetatoken.org/rpc`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: mandatoryConf.ETHERSCAN_API_KEY,
  },
};
