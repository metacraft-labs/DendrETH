import { Contract } from 'ethers';
import { groth16 } from 'snarkjs';

export const RELAYER_UPDATES_FOLDER = 'relayer_updates';
export const RELAYER_INPUTS_FOLDER = 'relayer_inputs';
export const RELAYER_WITNESSES_FOLDER = 'relayer_witnesses';
export const RELAYER_PROOFS_FOLDER = 'relayer_proofs';

export const UPDATE_POLING_QUEUE = 'update_poling';
export const WITNESS_GENERATOR_QUEUE = 'witness';
export const INPUT_GENERATOR_QUEUE = 'input';
export const PROOF_GENERATOR_QUEUE = 'proof';
export const PUBLISH_ONCHAIN_QUEUE = 'publish_on_chain';

export const PROOFS_CHANEL = 'proofs';

export const EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
export const SLOTS_PER_EPOCH = 32;

export function computeSyncCommitteePeriodAt(slot: number) {
  return Math.floor(
    slot / (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH),
  );
}

export function getProofKey(prevUpdateSlot: number, updateSlot: number) {
  return `proof:${prevUpdateSlot}:${updateSlot}`;
}

// TODO move to a separate file?
export async function postUpdateOnChain(
  proofResult: ProofResultType,
  lightClientContract: Contract,
) {
  const calldata = await groth16.exportSolidityCallData(
    proofResult.proof,
    proofResult.proof.public,
  );

  const argv: string[] = calldata
    .replace(/["[\]\s]/g, '')
    .split(',')
    .map(x => BigInt(x).toString());

  const a = [argv[0], argv[1]];
  const b = [
    [argv[2], argv[3]],
    [argv[4], argv[5]],
  ];
  const c = [argv[6], argv[7]];

  const transaction = await lightClientContract.light_client_update({
    attested_header_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.nextHeaderHash.join(''))
        .toString(16)
        .padStart(64, '0'),
    finalized_header_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.finalizedHeaderRoot.join(''))
        .toString(16)
        .padStart(64, '0'),
    finalized_execution_state_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.execution_state_root.join(''))
        .toString(16)
        .padStart(64, '0'),
    a: a,
    b: b,
    c: c,
  });

  console.log(transaction);

  await transaction.wait();
}

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
  lastDownloadedUpdateKey: string;
  slotsJump: number;
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
