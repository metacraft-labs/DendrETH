export const RELAYER_UPDATES_FOLDER = 'relayer_updates';
export const RELAYER_INPUTS_FOLDER = 'relayer_inputs';
export const RELAYER_WITNESSES_FOLDER = 'relayer_witnesses';
export const RELAYER_PROOFS_FOLDER = 'relayer_proofs';

export const UPDATE_POLING_QUEUE = 'update_poling';
export const WITNESS_GENERATOR_QUEUE = 'witness';
export const INPUT_GENERATOR_QUEUE = 'input';
export const PROOF_GENERATOR_QUEUE = 'proof';

export const EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
export const SLOTS_PER_EPOCH = 32;

export function computeSyncCommitteePeriodAt(slot: number) {
  return Math.floor(
    slot / (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH),
  );
}

export type ProofInputType = {
  prevUpdateSlot: number;
  updateSlot: number;
  proofInput: WitnessGeneratorInput;
};

export type State = {
  lastDownloadedUpdate: number;
  lastUpdateOnChain: number;
};

export interface Update {
  version: string;
  data: Data;
}

export interface Data {
  attested_header: Header;
  finalized_header: Header;
  finality_branch: string[];
  sync_aggregate: SyncAggregate;
  signature_slot: string;
}

export interface SyncCommittee {
  pubkeys: string[];
  aggregate_pubkey: string;
}

export interface Header {
  beacon: Beacon;
  execution: Execution;
  execution_branch: string[];
}

export interface Beacon {
  slot: string;
  proposer_index: string;
  parent_root: string;
  state_root: string;
  body_root: string;
}

export interface Execution {
  parent_hash: string;
  fee_recipient: string;
  state_root: string;
  receipts_root: string;
  logs_bloom: string;
  prev_randao: string;
  block_number: string;
  gas_limit: string;
  gas_used: string;
  timestamp: string;
  extra_data: string;
  base_fee_per_gas: string;
  block_hash: string;
  transactions_root: string;
  withdrawals_root: string;
}

export interface SyncAggregate {
  sync_committee_bits: string;
  sync_committee_signature: string;
}

export interface WitnessGeneratorInput {
  points: string[][][];
  signatureSlot: string;
  signatureSlotSyncCommitteePeriod: number;
  finalizedHeaderSlotSyncCommitteePeriod: number;
  prevHeaderHash: string[];
  nextHeaderHash: string[];
  prevHeaderStateRoot: string[];
  prevHeaderStateRootBranch: string[][];
  prevHeaderFinalizedSlotBranch: string[][];
  prevHeaderFinalizedSlot: number;
  nextHeaderSlotBranch: string[][];
  nextHeaderSlot: number;
  finalizedHeaderRoot: string[];
  finalizedHeaderBranch: string[][];
  execution_state_root: string[];
  execution_state_root_branch: string[][];
  fork_version: string[];
  GENESIS_VALIDATORS_ROOT: string[];
  DOMAIN_SYNC_COMMITTEE: string[];
  aggregatedKey: string[];
  syncCommitteeBranch: string[][];
  bitmask: string[];
  signature: string[][][];
}

export interface Proof {
  pi_a: string[];
  pi_b: string[][];
  pi_c: string[];
  public: string[];
}
