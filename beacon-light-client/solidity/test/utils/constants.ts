import { hexToBytes } from './bls';

export const SLOTS_PER_EPOCH: number = 32;

export const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: number = 256;

export const GENESIS_FORK_VERSION: Uint8Array = hexToBytes('0x00000000');

export const ALTAIR_FORK_VERSION: Uint8Array = hexToBytes('0x01000000');

export const DOMAIN_SYNC_COMMITTEE: Uint8Array = hexToBytes('0x07000000');

export const GENESIS_VALIDATORS_ROOT: Uint8Array = hexToBytes(
  '0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95',
);

export const DOMAIN_BEACON_PROPOSER = Uint8Array.from([0, 0, 0, 0]);
