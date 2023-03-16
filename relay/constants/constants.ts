import { hexToBytes } from '../../libs/typescript/ts-utils/bls';

export type Config = {
  SLOTS_PER_EPOCH: number;
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: number;
  GENESIS_FORK_VERSION: Uint8Array;
  FORK_VERSION: Uint8Array;
  DOMAIN_SYNC_COMMITTEE: Uint8Array;
  GENESIS_VALIDATORS_ROOT: Uint8Array;
};

export const PRATER: Config = {
  SLOTS_PER_EPOCH: 32,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
  GENESIS_FORK_VERSION: hexToBytes('0x00001020'),
  FORK_VERSION: hexToBytes('0x03001020'),
  DOMAIN_SYNC_COMMITTEE: hexToBytes('0x07000000'),
  GENESIS_VALIDATORS_ROOT: hexToBytes(
    '0x043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb',
  ),
};

export const UPDATE_POLING_QUEUE = 'update_poling';
export const PROOF_GENERATOR_QUEUE = 'proof';
