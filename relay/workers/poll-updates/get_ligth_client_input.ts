import {
  BitVectorType,
  ByteListType,
  ByteVectorType,
  ContainerType,
  JsonPath,
  UintBigintType,
  UintNumberType,
} from '@chainsafe/ssz';
import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '../../../libs/typescript/ts-utils/bls';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Config } from '../../constants/constants';
import {
  BeaconBlockHeader,
  ExecutionPayloadHeader,
  SyncAggregate,
  SyncCommittee,
  WitnessGeneratorInput,
} from '../../types/types';
import { computeSyncCommitteePeriodAt } from '../../../libs/typescript/ts-utils/ssz-utils';

const ExecutionPayload = new ContainerType({
  parentHash: new ByteVectorType(32),
  feeRecipient: new ByteVectorType(20),
  stateRoot: new ByteVectorType(32),
  receiptsRoot: new ByteVectorType(32),
  logsBloom: new ByteVectorType(256),
  prevRandao: new ByteVectorType(32),
  blockNumber: new UintNumberType(8),
  gasLimit: new UintNumberType(8),
  gasUsed: new UintNumberType(8),
  timestamp: new UintNumberType(8),
  extraData: new ByteListType(32),
  baseFeePerGas: new UintBigintType(32),
  blockHash: new ByteVectorType(32),
  transactionsRoot: new ByteVectorType(32),
  withdrawalsRoot: new ByteVectorType(32),
});

function getMerkleProof(
  container: ContainerType<any>,
  path: JsonPath,
  value: any,
) {
  const view = container.toViewDU(value);
  const tree = new Tree(view.node);
  const proof = tree.getSingleProof(container.getPathInfo(path).gindex);

  return proof.map(bytesToHex);
}

function hexToBits(hex: string, numbersOfBits = 256) {
  return BigInt('0x' + formatHex(hex))
    .toString(2)
    .padStart(numbersOfBits, '0')
    .split('');
}

export async function getProofInput({
  syncCommittee,
  syncCommitteeBranch,
  sync_aggregate,
  prevBlockHeader,
  nextBlockHeader,
  finalizedHeader,
  finalityBranch,
  finalizedHeaderExecutionBranch,
  prevFinalizedHeader,
  prevFinalityBranch,
  executionPayload,
  signature_slot,
  config,
}: {
  syncCommittee: SyncCommittee;
  syncCommitteeBranch: string[];
  sync_aggregate: SyncAggregate;
  prevBlockHeader: BeaconBlockHeader;
  nextBlockHeader: BeaconBlockHeader;
  finalizedHeader: BeaconBlockHeader;
  finalityBranch: string[];
  finalizedHeaderExecutionBranch: string[];
  prevFinalizedHeader: BeaconBlockHeader;
  prevFinalityBranch: string[];
  executionPayload: ExecutionPayloadHeader;
  signature_slot: number;
  config: Config;
}): Promise<WitnessGeneratorInput> {
  const { ssz } = await import('@lodestar/types');

  let syncCommitteePubkeys: PointG1[] = syncCommittee.pubkeys.map(x =>
    PointG1.fromHex(x.slice(2)),
  );

  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(sync_aggregate.sync_committee_bits);

  let signature: PointG2 = PointG2.fromSignature(
    formatHex(sync_aggregate.sync_committee_signature),
  );

  const prevBlockHeaderStateRootProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['state_root'],
    prevBlockHeader,
  ).map(x => hexToBits(x));

  const prevBlockHeaderHash =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(prevBlockHeader);

  const nextBlockHeaderHash =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(nextBlockHeader);
  const nextBlockHeaderStateRootProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['state_root'],
    nextBlockHeader,
  ).map(x => hexToBits(x));

  let nextHeaderSlotBranch = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['slot'],
    nextBlockHeader,
  ).map(x => hexToBits(x));

  let syncCommitteeBranchBits = syncCommitteeBranch.map(x => hexToBits(x));

  let finalizedHeaderHash =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedHeader);

  let finalityBranchBits = finalityBranch.map(x => hexToBits(x));

  let finalizedHeaderBodyRootProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['body_root'],
    finalizedHeader,
  );

  let prevHeaderFinalizedBranch = prevFinalityBranch.map(x => hexToBits(x));

  let prevHeaderFinalizedSlotBranch = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['slot'],
    prevFinalizedHeader,
  ).map(x => hexToBits(x));

  let prevHeaderFinalizedRoot = hexToBits(
    bytesToHex(ssz.phase0.BeaconBlockHeader.hashTreeRoot(prevFinalizedHeader)),
  );

  let prevFinalizedHeaderStateProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['state_root'],
    prevFinalizedHeader,
  ).map(x => hexToBits(x));

  const executionPayloadStateProof = getMerkleProof(
    ExecutionPayload,
    ['stateRoot'],
    executionPayload,
  );

  let dataView = new DataView(new ArrayBuffer(8));
  dataView.setBigUint64(0, BigInt(finalizedHeader.proposerIndex));
  let proposer_index = dataView.getBigUint64(0, true);
  proposer_index = BigInt(
    '0x' + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'),
  );

  return {
    prevHeaderHash: hexToBits(bytesToHex(prevBlockHeaderHash)),
    nextHeaderHash: hexToBits(bytesToHex(nextBlockHeaderHash)),
    prevFinalizedHeaderRoot: prevHeaderFinalizedRoot,
    prevFinalizedHeaderRootBranch: [
      ...prevHeaderFinalizedBranch,
      ...prevBlockHeaderStateRootProof,
    ],
    prevHeaderFinalizedStateRoot: hexToBits(
      bytesToHex(prevFinalizedHeader.stateRoot),
    ),

    prevHeaderFinalizedStateRootBranch: prevFinalizedHeaderStateProof,

    prevHeaderFinalizedSlot: prevFinalizedHeader.slot,
    prevHeaderFinalizedSlotBranch: [...prevHeaderFinalizedSlotBranch],
    nextHeaderSlot: nextBlockHeader.slot,
    nextHeaderSlotBranch: nextHeaderSlotBranch,

    signatureSlot: signature_slot.toString(),

    signatureSlotSyncCommitteePeriod:
      computeSyncCommitteePeriodAt(signature_slot),
    finalizedHeaderSlotSyncCommitteePeriod: computeSyncCommitteePeriodAt(
      prevFinalizedHeader.slot,
    ),
    finalizedHeaderRoot: hexToBits(bytesToHex(finalizedHeaderHash)),
    finalizedHeaderBranch: [
      ...finalityBranchBits,
      ...nextBlockHeaderStateRootProof,
    ],

    execution_state_root: hexToBits(bytesToHex(executionPayload.stateRoot)),
    execution_state_root_branch: [
      ...executionPayloadStateProof,
      ...finalizedHeaderExecutionBranch,
      ...finalizedHeaderBodyRootProof,
    ].map(x => hexToBits(x)),

    fork_version: hexToBits(bytesToHex(config.FORK_VERSION), 32),
    GENESIS_VALIDATORS_ROOT: hexToBits(
      bytesToHex(config.GENESIS_VALIDATORS_ROOT),
    ),
    DOMAIN_SYNC_COMMITTEE: hexToBits(
      bytesToHex(config.DOMAIN_SYNC_COMMITTEE),
      32,
    ),

    points: syncCommitteePubkeys.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
    aggregatedKey: hexToBits(syncCommittee.aggregate_pubkey, 384),
    syncCommitteeBranch: [...syncCommitteeBranchBits],

    bitmask: bitmask.toBoolArray().map(x => (x ? '1' : '0')),
    signature: [
      [
        bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[0].c1.value),
      ],
      [
        bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[1].c1.value),
      ],
    ],
  };
}
