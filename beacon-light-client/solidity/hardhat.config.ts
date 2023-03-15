require('dotenv').config();

import 'hardhat-gas-reporter';

import '@nomiclabs/hardhat-waffle';
import '@nomiclabs/hardhat-etherscan';
import '@nomiclabs/hardhat-ethers';

import './tasks';
import { HardhatConfig } from 'hardhat/types';

const optionalConf = {
  USER_PRIVATE_KEY: process.env.USER_PRIVATE_KEY,
  ALCHEMY_API_KEY: process.env.ALCHEMY_API_KEY,
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

const commonConfig = {
  rapidSnarkProverPath: '../../../../../vendor/rapidsnark/build/prover',
  zkeyFilePath: '../../../build/light_client/light_client_0.zkey',
  witnessGeneratorPath:
    '../../../build/light_client/light_client_cpp/light_client',
  slotsJump: 32,
  startingSlot: 294811,
  redisHost: 'localhost',
  redisPort: 6379,
};

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
      beaconApi: 'http://192.168.1.116:4000',
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
      url: `https://sepolia.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      accounts: [optionalConf.USER_PRIVATE_KEY],
      contractAddress: '0xA3418F79c98A3E496A5E97610a97f82daE364619',
    },
    goerli: {
      url: `https://goerli.infura.io/v3/${mandatoryConf.INFURA_API_KEY}`,
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      accounts: [optionalConf.USER_PRIVATE_KEY],
      contractAddress: '0xFb3Bb7992A49703D4f3AEAA2FA95AA250aBE2936',
      ...commonConfig,
    },
    optimisticGoerli: {
      url: `https://opt-goerli.g.alchemy.com/v2/${optionalConf.ALCHEMY_API_KEY}`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      contractAddress: '0x1a2FAA5f49385EebA349fd2616BAbf1Eb4367dcc',
      ...commonConfig,
    },
    baseGoerli: {
      url: 'https://base-goerli.rpc.thirdweb.com',
      accounts: [optionalConf.USER_PRIVATE_KEY],
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      contractAddress: '0xB94868ba0903883bD2dE3311Fc377f3c50D602eA',
      ...commonConfig,
    },
    arbitrumGoerli: {
      url: `https://arb-goerli.g.alchemy.com/v2/F6HTzShSaOutFCEw4DzMZRIYowlm6Hap`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      contractAddress: '0xB94868ba0903883bD2dE3311Fc377f3c50D602eA',
      ...commonConfig,
    },
    mumbai: {
      url: `https://polygon-mumbai.g.alchemy.com/v2/TQfySrJXZGI_1HMYhCvI5DepLpHcgtHy`,
      accounts: [optionalConf.USER_PRIVATE_KEY],
      beaconApi: 'http://unstable.prater.beacon-api.nimbus.team',
      contractAddress: '0xA3418F79c98A3E496A5E97610a97f82daE364619',
      ...commonConfig,
    },
  },
  mocha: {
    timeout: 100000000,
  },
  etherscan: {
    apiKey: mandatoryConf.ETHERSCAN_API_KEY,
    customChains: [
      {
        network: 'baseGoerli',
        chainId: 84531,
        urls: {
          apiURL: 'https://goerli.basescan.org/api',
          browserURL: 'https://goerli.basescan.org',
        },
      },
    ],
  },
};
