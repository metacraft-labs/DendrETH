import '@nomiclabs/hardhat-waffle';
import { HardhatUserConfig } from 'hardhat/types';

const config: HardhatUserConfig = {
  solidity: '0.8.9',
  mocha: {
    timeout: 100000000,
  },
};

export default config;
