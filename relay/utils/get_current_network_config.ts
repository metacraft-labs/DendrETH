import { BeaconApi, getBeaconApi } from '@/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';

interface Config {
  NETWORK_NAME: string;
  BEACON_REST_API: string[];
  SLOTS_PER_EPOCH: number;
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: number;
  GENESIS_FORK_VERSION: string;
  FORK_VERSION: string;
  DOMAIN_SYNC_COMMITTEE: string;
  GENESIS_VALIDATORS_ROOT: string;
}

const defaultConfig: Config = {
  NETWORK_NAME: '',
  BEACON_REST_API: [],
  SLOTS_PER_EPOCH: 0,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 0,
  GENESIS_FORK_VERSION: '',
  FORK_VERSION: '',
  DOMAIN_SYNC_COMMITTEE: '',
  GENESIS_VALIDATORS_ROOT: '',
};
export enum NetworkConfig {
  Pratter = 'pratter',
  Mainnet = 'mainnet',
  Sepolia = 'sepolia',
  Chiado = 'chiado',
}

export function isSupportedFollowNetwork(network: string): boolean {
  return Object.values(NetworkConfig).includes(network as NetworkConfig);
}

export async function getNetworkConfig(
  network: NetworkConfig,
): Promise<Config> {
  let config: Config = { ...defaultConfig, NETWORK_NAME: network };
  config.NETWORK_NAME = network;
  switch (network) {
    case 'pratter': {
      config.BEACON_REST_API[0] =
        process.env.BEACON_REST_API_PRATER || 'default_prater_rest_api_url';
      break;
    }
    case 'mainnet': {
      config.BEACON_REST_API[0] =
        process.env.BEACON_REST_API_MAINNET || 'default_mainnet_rest_api_url';
      break;
    }
    case 'sepolia': {
      config.BEACON_REST_API[0] =
        process.env.BEACON_REST_API_SEPOLIA || 'default_sepolia_rest_api_url';
      break;
    }
    case 'chiado': {
      config.BEACON_REST_API[0] =
        process.env.BEACON_REST_API_CHIADO || 'default_chiado_rest_api_url';
      break;
    }
    default: {
      throw new Error('Network not supported');
      break;
    }
  }

  const beaconApi = await getBeaconApi(config.BEACON_REST_API);

  const specConfig = await beaconApi.getSpecConfig();

  const genesisConfig = await beaconApi.getGenesisData();

  config.SLOTS_PER_EPOCH = specConfig.SLOTS_PER_EPOCH;
  config.EPOCHS_PER_SYNC_COMMITTEE_PERIOD =
    specConfig.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;
  config.GENESIS_FORK_VERSION = specConfig.GENESIS_FORK_VERSION;
  config.FORK_VERSION = specConfig.DENEB_FORK_VERSION;
  config.DOMAIN_SYNC_COMMITTEE = specConfig.DOMAIN_SYNC_COMMITTEE;
  config.GENESIS_VALIDATORS_ROOT = bytesToHex(
    genesisConfig.genesisValidatorsRoot,
  );

  return config;
}
