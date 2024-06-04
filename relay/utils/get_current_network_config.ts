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

export async function getNetworkConfig(
  network: 'pratter' | 'mainnet',
): Promise<Config> {
  let config: Config = { ...defaultConfig, NETWORK_NAME: network };
  config.NETWORK_NAME = network;
  config.BEACON_REST_API[0] =
    network === 'mainnet'
      ? process.env.BEACON_REST_API_MAINNET ?? 'default_mainnet_rest_api_url'
      : process.env.BEACON_REST_API_PRATER ?? 'default_prater_rest_api_url';

  const response = await fetch(config.BEACON_REST_API + '/eth/v1/config/spec');
  if (!response.ok) {
    throw new Error('Network response was not ok ' + response.statusText);
  }
  const responseGenesis = await fetch(
    config.BEACON_REST_API + '/eth/v1/beacon/genesis',
  );
  if (!responseGenesis.ok) {
    throw new Error(
      'Network response was not ok ' + responseGenesis.statusText,
    );
  }
  const config_ = await response.json();
  const config_genesis = await responseGenesis.json();

  config.SLOTS_PER_EPOCH = config_.data.SLOTS_PER_EPOCH;
  config.EPOCHS_PER_SYNC_COMMITTEE_PERIOD =
    config_.data.EPOCHS_PER_SYNC_COMMITTEE_PERIOD;
  config.GENESIS_FORK_VERSION = config_.data.GENESIS_FORK_VERSION;
  config.FORK_VERSION = config_.data.DENEB_FORK_VERSION;
  config.DOMAIN_SYNC_COMMITTEE = config_.data.DOMAIN_SYNC_COMMITTEE;
  config.GENESIS_VALIDATORS_ROOT = config_genesis.data.genesis_validators_root;

  return config;
}
