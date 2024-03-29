export type Config = {
  SLOTS_PER_EPOCH: number;
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: number;
  GENESIS_FORK_VERSION: string;
  FORK_VERSION: string;
  DOMAIN_SYNC_COMMITTEE: string;
  GENESIS_VALIDATORS_ROOT: string;
  BEACON_REST_API: string[];
};

export const PRATER: Config = {
  SLOTS_PER_EPOCH: 32,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
  GENESIS_FORK_VERSION: '0x00001020',
  FORK_VERSION: '0x03001020',
  DOMAIN_SYNC_COMMITTEE: '0x07000000',
  GENESIS_VALIDATORS_ROOT:
    '0x043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb',
  BEACON_REST_API: [
    'https://purple-falling-tree.ethereum-goerli.discover.quiknode.pro/',
  ],
};

export const UPDATE_POLING_QUEUE = 'update_poling';
export const PROOF_GENERATOR_QUEUE = 'proof';
