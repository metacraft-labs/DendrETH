require('dotenv').config();

import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-ethers';

import './tasks';

const conf = {
  USER_PRIVATE_KEY: process.env.USER_PRIVATE_KEY,
  ALCHEMY_API_KEY: process.env.ALCHEMY_API_KEY,
  BASE_ETHERSCAN_API_KEY: process.env.BASE_ETHERSCAN_API_KEY,
  POLYGON_MUMBAI_ETHERSCAN_API_KEY:
    process.env.POLYGON_MUMBAI_ETHERSCAN_API_KEY,
  ARBITRUM_ETHERSCAN_API_KEY: process.env.ARBITRUM_ETHERSCAN_API_KEY,
  OPTIMISM_ETHERSCAN_API_KEY: process.env.OPTIMISM_ETHERSCAN_API_KEY,
  AVALANCHE_FUJI_ETHERSCAN_API_KEY:
    process.env.AVALANCHE_FUJI_ETHERSCAN_API_KEY,
  FTM_ETHERSCAN_API_KEY: process.env.FTM_ETHERSCAN_API_KEY,
  CELO_ETHERSCAN_API_KEY: process.env.CELO_ETHERSCAN_API_KEY,
  BSC_ETHERSCAN_API_KEY: process.env.BSC_ETHERSCAN_API_KEY,
  CHIADO_ETHERSCAN_API: process.env.CHIADO_ETHERSCAN_API,
  LOCAL_HARDHAT_PRIVATE_KEY:
    process.env.LOCAL_HARDHAT_PRIVATE_KEY ||
    '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
  INFURA_API_KEY: process.env.INFURA_API_KEY,
  ETHERSCAN_API_KEY: process.env.ETHERSCAN_API_KEY,
};

if (!/^0x[0-9a-fA-F]{64}$/.test(conf.USER_PRIVATE_KEY ?? '')) {
  console.warn(
    'Setting $USER_PRIVATE_KEY to $LOCAL_HARDHAT_PRIVATE_KEY as fallback',
  );
  conf.USER_PRIVATE_KEY = conf.LOCAL_HARDHAT_PRIVATE_KEY;
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
      accounts: [conf.LOCAL_HARDHAT_PRIVATE_KEY],
    },
    hardhat: {
      blockGasLimit: 30000000,
      allowUnlimitedContractSize: true,
    },
    ropsten: {
      url: `https://ropsten.infura.io/v3/${conf.INFURA_API_KEY}`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    sepolia: {
      url: `https://eth-sepolia.g.alchemy.com/v2/${conf.ALCHEMY_API_KEY}`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    goerli: {
      url: `https://eth-goerli.g.alchemy.com/v2/${conf.ALCHEMY_API_KEY}`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    optimisticGoerli: {
      url: `https://opt-goerli.g.alchemy.com/v2/${conf.ALCHEMY_API_KEY}`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    baseGoerli: {
      url: 'https://base-goerli.public.blastapi.io',
      accounts: [conf.USER_PRIVATE_KEY],
    },
    arbitrumGoerli: {
      url: `https://arb-goerli.g.alchemy.com/v2/${conf.ALCHEMY_API_KEY}`,
      accounts: [conf.USER_PRIVATE_KEY],
      contractAddress: '0xB94868ba0903883bD2dE3311Fc377f3c50D602eA',
    },
    mumbai: {
      url: `https://endpoints.omniatech.io/v1/matic/mumbai/public`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    avalanche: {
      url: `https://rpc.ankr.com/avalanche_fuji`,
      accounts: [conf.USER_PRIVATE_KEY],
    },
    fantom: {
      url: 'https://rpc.testnet.fantom.network',
      accounts: [conf.USER_PRIVATE_KEY],
    },
    celo: {
      url: 'https://alfajores-forno.celo-testnet.org',
      accounts: [conf.USER_PRIVATE_KEY],
    },
    bsc: {
      url: 'https://bsc-testnet.public.blastapi.io',
      accounts: [conf.USER_PRIVATE_KEY],
    },
    chiado: {
      url: 'https://rpc.chiado.gnosis.gateway.fm',
      accounts: [conf.USER_PRIVATE_KEY],
      gas: 30000000,
      gasPrice: 20,
      gasMultiplier: 10,
    },
    evmos: {
      url: 'https://eth.bd.evmos.dev:8545',
      accounts: [conf.USER_PRIVATE_KEY],
    },
    aurora: {
      url: 'https://aurora-testnet.rpc.thirdweb.com',
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
