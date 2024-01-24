import {
  UintNumberType,
  ByteVectorType,
  ByteListType,
  UintBigintType,
} from '@chainsafe/ssz';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { Config } from '../constants/constants';

export type BeaconBlockHeader = ValueOfFields<{
  slot: UintNumberType;
  proposerIndex: UintNumberType;
  parentRoot: ByteVectorType;
  stateRoot: ByteVectorType;
  bodyRoot: ByteVectorType;
}>;

export type ExecutionPayloadHeader = ValueOfFields<{
  withdrawalsRoot: ByteVectorType;
  transactionsRoot: ByteVectorType;
  blockHash: ByteVectorType;
  parentHash: ByteVectorType;
  feeRecipient: ByteVectorType;
  stateRoot: ByteVectorType;
  receiptsRoot: ByteVectorType;
  logsBloom: ByteVectorType;
  prevRandao: ByteVectorType;
  blockNumber: UintNumberType;
  gasLimit: UintNumberType;
  gasUsed: UintNumberType;
  timestamp: UintNumberType;
  extraData: ByteListType;
  baseFeePerGas: UintBigintType;
}>;

export type ProofInputType = {
  prevUpdateSlot: number;
  updateSlot: number;
  proofInput: WitnessGeneratorInput;
};

export type ProofResultType = {
  prevUpdateSlot: number;
  updateSlot: number;
  proof: Proof;
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

export interface UpdateResult {
  prevUpdateSlot: number;
  updateSlot: number;
}

export interface GetUpdate {
  from: number;
  to: number;
  networkConfig: Config;
}

export interface WitnessGeneratorInput {
  prevFinalizedHeaderRoot: string[];
  prevFinalizedHeaderRootBranch: string[][];
  prevHeaderFinalizedStateRoot: string[];
  prevHeaderFinalizedStateRootBranch: string[][];
  points: string[][][];
  signatureSlot: string;
  signatureSlotSyncCommitteePeriod: number;
  finalizedHeaderSlotSyncCommitteePeriod: number;
  prevHeaderHash: string[];
  nextHeaderHash: string[];
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

export interface Validator {
  pubkey: Uint8Array;
  withdrawalCredentials: Uint8Array;
  effectiveBalance: number;
  slashed: boolean;
  activationEligibilityEpoch: number;
  activationEpoch: number;
  exitEpoch: number;
  withdrawableEpoch: number;
}

export interface ValidatorProof {
  needsChange: boolean;
  proof: number[];
  poseidonHash: number[];
  sha256Hash: number[];
}

export interface BalanceProof {
  needsChange: boolean;
  rangeTotalValue: string;
  validatorsCommitment: number[];
  proof: number[];
  balancesHash: number[];
  withdrawalCredentials: string;
  currentEpoch: string;
}

export interface IndexedValidator {
  index: number;
  validator: Validator;
}
