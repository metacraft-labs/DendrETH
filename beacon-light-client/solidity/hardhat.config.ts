require('dotenv').config();

import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-ethers';

import './tasks';

const envConfig = {
  USER_PRIVATE_KEY: process.env.USER_PRIVATE_KEY,
  LOCAL_HARDHAT_PRIVATE_KEY: process.env.LOCAL_HARDHAT_PRIVATE_KEY,
  INFURA_API_KEY: process.env.INFURA_API_KEY,
  ETHERSCAN_API_KEY: process.env.ETHERSCAN_API_KEY,
};

for (const envVar of Object.keys(envConfig)) {
  if (!envConfig[envVar]) {
    console.warn(`$${envVar} environment variable is not set`);
    process.exit(0);
  }
}

if (!/^0x[0-9a-fA-F]{64}$/.test(envConfig.USER_PRIVATE_KEY ?? '')) {
  console.warn(
    'Setting $USER_PRIVATE_KEY to $LOCAL_HARDHAT_PRIVATE_KEY as fallback',
  );
  envConfig.USER_PRIVATE_KEY = envConfig.LOCAL_HARDHAT_PRIVATE_KEY;
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
      accounts: [envConfig.LOCAL_HARDHAT_PRIVATE_KEY],
    },
    hardhat: {
      blockGasLimit: 30000000,
      allowUnlimitedContractSize: true,
    },
    ropsten: {
      url: `https://ropsten.infura.io/v3/${envConfig.INFURA_API_KEY}`,
      accounts: [envConfig.USER_PRIVATE_KEY],
    },
    mainnet: {
      url: `https://goerli.infura.io/v3/${envConfig.INFURA_API_KEY}`,
      accounts: [envConfig.USER_PRIVATE_KEY],
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: envConfig.ETHERSCAN_API_KEY,
  },
};
