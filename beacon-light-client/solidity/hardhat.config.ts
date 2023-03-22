require('dotenv').config();

import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-ethers';

import './tasks';

const optionalConf = {
  USER_PRIVATE_KEY: process.env.USER_PRIVATE_KEY,
  ALCHEMY_API_KEY: process.env.ALCHEMY_API_KEY,
  BASE_ETHERSCAN_API_KEY: process.env.BASE_ETHERSCAN_API_KEY,
  POLYGON_MUMBAI_ETHERSCAN_API_KEY:
    process.env.POLYGON_MUMBAI_ETHERSCAN_API_KEY,
  ARBITRUM_ETHERSCAN_API_KEY: process.env.ARBITRUM_ETHERSCAN_API_KEY,
  OPTIMISM_ETHERSCAN_API_KEY: process.env.OPTIMISM_ETHERSCAN_API_KEY,
};

const mandatoryConf = {
  LOCAL_HARDHAT_PRIVATE_KEY:
    process.env.LOCAL_HARDHAT_PRIVATE_KEY ||
    '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
  INFURA_API_KEY: process.env.INFURA_API_KEY,
  ETHERSCAN_API_KEY: process.env.ETHERSCAN_API_KEY,
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
      url: 'http://127.0.0.1:8545',
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
    sepolia: {
      url: `https://eth-sepolia.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    goerli: {
      url: `https://eth-goerli.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    optimisticGoerli: {
      url: `https://opt-goerli.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    baseGoerli: {
      url: 'https://base-goerli.rpc.thirdweb.com',
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
    arbitrumGoerli: {
      url: `https://arb-goerli.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
      contractAddress: '0xB94868ba0903883bD2dE3311Fc377f3c50D602eA',
    },
    mumbai: {
      url: `https://polygon-mumbai.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: {
      optimisticGoerli: optionalConf.OPTIMISM_ETHERSCAN_API_KEY,
      arbitrumGoerli: optionalConf.ARBITRUM_ETHERSCAN_API_KEY,
      polygonMumbai: optionalConf.POLYGON_MUMBAI_ETHERSCAN_API_KEY,
      baseGoerli: optionalConf.BASE_ETHERSCAN_API_KEY,
      sepolia: mandatoryConf.ETHERSCAN_API_KEY,
      goerli: mandatoryConf.ETHERSCAN_API_KEY,
    },
    customChains: [
      {
        network: 'baseGoerli',
        chainId: 84531,
        urls: {
          apiURL: 'https://api-goerli.basescan.org/api',
          browserURL: 'https://goerli.basescan.org',
        },
      },
      {
        network: 'optimisticGoerli',
        chainId: 420,
        urls: {
          apiURL: 'https://api-goerli-optimism.etherscan.io/api',
          browserURL: 'https://goerli-optimism.etherscan.io',
        },
      },
    ],
  },
};
