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

export type NetworkConfig =
  | 'pratter'
  | 'mainnet'
  | 'sepolia'
  | 'chiado'
  | 'gnosis';

export function isSupportedFollowNetwork(
  network: string,
): network is NetworkConfig {
  return ['pratter', 'mainnet', 'sepolia', 'chiado', 'gnosis'].includes(
    network,
  );
}

export async function getNetworkConfig(
  network: NetworkConfig,
): Promise<Config> {
  let config: Config = { ...defaultConfig, NETWORK_NAME: network };
  config.NETWORK_NAME = network;

  const envVarName = `BEACON_REST_API_${network.toUpperCase()}`;
  const envVarValue = process.env[envVarName];

  if (!envVarValue) {
    throw new Error(`${envVarName} is not defined`);
  }

  config.BEACON_REST_API = envVarValue.split(',');

  const beaconApi = await getBeaconApi(config.BEACON_REST_API);

  const config_genesis = await beaconApi.getGenesisData();

  config.SLOTS_PER_EPOCH = Number(await beaconApi.getSlotsPerEpoch());
  config.EPOCHS_PER_SYNC_COMMITTEE_PERIOD = Number(
    await beaconApi.getSlotsPerSyncCommitteePeriod(),
  );
  config.GENESIS_FORK_VERSION = bytesToHex(config_genesis.genesisForkVersion);
  config.FORK_VERSION = await beaconApi.getForkVersion();
  config.DOMAIN_SYNC_COMMITTEE = await beaconApi.getDomainSyncCommittee();
  config.GENESIS_VALIDATORS_ROOT = bytesToHex(
    config_genesis.genesisValidatorsRoot,
  );

  return config;
}
