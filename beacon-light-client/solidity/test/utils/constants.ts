import { hexToBytes } from './bls';

export type Config = {
  SLOTS_PER_EPOCH: number;
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: number;
  GENESIS_FORK_VERSION: Uint8Array;
  FORK_VERSION: Uint8Array;
  DOMAIN_SYNC_COMMITTEE: Uint8Array;
  GENESIS_VALIDATORS_ROOT: Uint8Array;
};

export const MAINNET: Config = {
  SLOTS_PER_EPOCH: 32,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
  GENESIS_FORK_VERSION: hexToBytes('0x00000000'),
  FORK_VERSION: hexToBytes('0x00000069'),
  DOMAIN_SYNC_COMMITTEE: hexToBytes('0x07000000'),
  GENESIS_VALIDATORS_ROOT: hexToBytes(
    '0x53a92d8f2bb1d85f62d16a156e6ebcd1bcaba652d0900b2c2f387826f3481f6f',
  ),
};

export const PRATER = {
  SLOTS_PER_EPOCH: 32,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
  GENESIS_FORK_VERSION: hexToBytes('0x00000000'),
  FORK_VERSION: hexToBytes('0x00000072'),
  DOMAIN_SYNC_COMMITTEE: hexToBytes('0x07000000'),
  GENESIS_VALIDATORS_ROOT: hexToBytes(
    '0x53a92d8f2bb1d85f62d16a156e6ebcd1bcaba652d0900b2c2f387826f3481f6f',
  ),
};

export const ZHEAJIANG_TESNET = {
  SLOTS_PER_EPOCH: 32,
  EPOCHS_PER_SYNC_COMMITTEE_PERIOD: 256,
  GENESIS_FORK_VERSION: hexToBytes('0x00000069'),
  FORK_VERSION: hexToBytes('0x00000072'),
  DOMAIN_SYNC_COMMITTEE: hexToBytes('0x07000000'),
  GENESIS_VALIDATORS_ROOT: hexToBytes(
    '0x53a92d8f2bb1d85f62d16a156e6ebcd1bcaba652d0900b2c2f387826f3481f6f',
  ),
};

export const DOMAIN_BEACON_PROPOSER = Uint8Array.from([0, 0, 0, 0]);
