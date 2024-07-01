import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Config } from '@/constants/constants';
import * as network_config from '@/constants/network_config.json';

export function getNetworkConfig(network: 'pratter' | 'mainnet'): Config {
  const config = {
    BEACON_REST_API_PRATTER: process.env.BEACON_REST_API_PRATTER,
    BEACON_REST_API_MAINNET: process.env.BEACON_REST_API_MAINNET,
  };

  checkConfig(config);

  network_config[network]['BEACON_REST_API'] =
    network === 'pratter'
      ? config.BEACON_REST_API_PRATTER!.split(',')
      : config.BEACON_REST_API_MAINNET!.split(',');

  return network_config[network] as Config;
}
