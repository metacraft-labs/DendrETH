require('dotenv').config();

require('tsconfig-paths').register({
  baseUrl: './',
  paths: {
    '@/*': ['../../relay/*'],
  },
});

// Disabled due to compatibility issues with pnpapi
// TODO: Replace with modern alternative
// import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-ethers';

import './tasks';
import { getSecretEnvString } from '@dendreth/utils/ts-utils/common-utils';

const conf = {
  USER_PRIVATE_KEY: getSecretEnvString('USER_PRIVATE_KEY_FILE'),
  ALCHEMY_API_KEY: getSecretEnvString('ALCHEMY_API_KEY_FILE'),
  BASE_ETHERSCAN_API_KEY: getSecretEnvString('BASE_ETHERSCAN_API_KEY_FILE'),
  POLYGON_MUMBAI_ETHERSCAN_API_KEY: getSecretEnvString(
    'POLYGON_MUMBAI_ETHERSCAN_API_KEY_FILE',
  ),
  ARBITRUM_ETHERSCAN_API_KEY: getSecretEnvString(
    'ARBITRUM_ETHERSCAN_API_KEY_FILE',
  ),
  OPTIMISM_ETHERSCAN_API_KEY: getSecretEnvString(
    'OPTIMISM_ETHERSCAN_API_KEY_FILE',
  ),
  AVALANCHE_FUJI_ETHERSCAN_API_KEY: getSecretEnvString(
    'AVALANCHE_FUJI_ETHERSCAN_API_KEY_FILE',
  ),
  FTM_ETHERSCAN_API_KEY: getSecretEnvString('FTM_ETHERSCAN_API_KEY_FILE'),
  CELO_ETHERSCAN_API_KEY: getSecretEnvString('CELO_ETHERSCAN_API_KEY_FILE'),
  BSC_ETHERSCAN_API_KEY: getSecretEnvString('BSC_ETHERSCAN_API_KEY_FILE'),
  CHIADO_ETHERSCAN_API: getSecretEnvString('CHIADO_ETHERSCAN_API_FILE'),
  GNOSIS_ETHERSCAN_API: getSecretEnvString('GNOSIS_ETHERSCAN_API_FILE'),
  LOCAL_HARDHAT_PRIVATE_KEY:
    getSecretEnvString('LOCAL_HARDHAT_PRIVATE_KEY_FILE') ||
    '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
  INFURA_API_KEY: getSecretEnvString('INFURA_API_KEY_FILE'),
  ETHERSCAN_API_KEY: getSecretEnvString('ETHERSCAN_API_KEY_FILE'),
  ETHEREUM_MAINNET_RPC: process.env.ETHEREUM_MAINNET_RPC || '',
  ROPSTEN_RPC: process.env.ROPSTEN_RPC || '',
  SEPOLIA_RPC: process.env.SEPOLIA_RPC || '',
  GOERLI_RPC: process.env.GOERLI_RPC || '',
  OPTIMISTIC_GOERLI_RPC: process.env.OPTIMISTIC_GOERLI_RPC || '',
  BASE_GOERLI_RPC: process.env.BASE_GOERLI_RPC || '',
  ARBITRUM_GOERLI_RPC: process.env.ARBITRUM_GOERLI_RPC || '',
  MUMBAI_RPC: process.env.MUMBAI_RPC || '',
  AVALANCHE_RPC: process.env.AVALANCHE_RPC || '',
  FANTOM_RPC: process.env.FANTOM_RPC || '',
  CELO_RPC: process.env.CELO_RPC || '',
  BSC_RPC: process.env.BSC_RPC || '',
  CHIADO_RPC: process.env.CHIADO_RPC || '',
  GNOSIS_RPC: process.env.GNOSIS_RPC || '',
  EVMOS_RPC: process.env.EVMOS_RPC || '',
  AURORA_RPC: process.env.AURORA_RPC || '',
  LUKSO_RPC: process.env.LUKSO_RPC || '',
};

if (!/^0x[0-9a-fA-F]{64}$/.test(conf.USER_PRIVATE_KEY ?? '')) {
  console.warn(
    'Setting $USER_PRIVATE_KEY to $LOCAL_HARDHAT_PRIVATE_KEY as fallback',
  );
  conf.USER_PRIVATE_KEY = conf.LOCAL_HARDHAT_PRIVATE_KEY;
}

export default {
  solidity: {
    compilers: [
      {
        version: '0.8.9',
        settings: {
          viaIR: true,
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
      {
        version: '0.8.18',
        settings: {
          viaIR: true,
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
      {
        version: '0.8.20',
        settings: {
          viaIR: true,
          optimizer: {
            enabled: true,
            runs: 200,
          },
        },
      },
    ],
  },
  defaultNetwork: 'hardhat',
  networks: {
    local: {
      url: 'http://127.0.0.1:8545',
      accounts: [conf.LOCAL_HARDHAT_PRIVATE_KEY],
    },
    hardhat: {
      forking: {
        url: conf.ETHEREUM_MAINNET_RPC,
        blockNumber: 19931139,
      },
    },
    ropsten: {
      url: conf.ROPSTEN_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    sepolia: {
      url: conf.SEPOLIA_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    goerli: {
      url: conf.GOERLI_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    optimisticGoerli: {
      url: conf.OPTIMISTIC_GOERLI_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    baseGoerli: {
      url: conf.BASE_GOERLI_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    arbitrumGoerli: {
      url: conf.ARBITRUM_GOERLI_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    mumbai: {
      url: conf.MUMBAI_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    avalanche: {
      url: conf.AVALANCHE_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    fantom: {
      url: conf.FANTOM_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    celo: {
      url: conf.CELO_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    bsc: {
      url: conf.BSC_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    chiado: {
      url: conf.CHIADO_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
      gas: 30000000,
      gasPrice: 20,
      gasMultiplier: 10,
    },
    evmos: {
      url: conf.EVMOS_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    aurora: {
      url: conf.AURORA_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    gnosis: {
      url: conf.GNOSIS_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    lukso: {
      url: conf.LUKSO_RPC,
      accounts: [conf.USER_PRIVATE_KEY],
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: {
      optimisticGoerli: conf.OPTIMISM_ETHERSCAN_API_KEY,
      arbitrumGoerli: conf.ARBITRUM_ETHERSCAN_API_KEY,
      polygonMumbai: conf.POLYGON_MUMBAI_ETHERSCAN_API_KEY,
      baseGoerli: conf.BASE_ETHERSCAN_API_KEY,
      sepolia: conf.ETHERSCAN_API_KEY,
      goerli: conf.ETHERSCAN_API_KEY,
      avalancheFujiTestnet: conf.AVALANCHE_FUJI_ETHERSCAN_API_KEY,
      ftmTestnet: conf.FTM_ETHERSCAN_API_KEY,
      celo: conf.CELO_ETHERSCAN_API_KEY,
      bscTestnet: conf.BSC_ETHERSCAN_API_KEY,
      chiado: conf.CHIADO_ETHERSCAN_API,
      gnosis: conf.GNOSIS_ETHERSCAN_API,
    },
    customChains: [
      {
        network: 'chiado',
        chainId: 10200,
        urls: {
          apiURL: 'https://gnosis-chiado.blockscout.com/api',
          browserURL: 'https://gnosis-chiado.blockscout.com/',
        },
      },
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
      {
        network: 'celo',
        chainId: 44787,
        urls: {
          apiURL: 'https://api-alfajores.celoscan.io/api',
          browserURL: 'https://alfajores.celoscan.io',
        },
      },
    ],
  },
};
