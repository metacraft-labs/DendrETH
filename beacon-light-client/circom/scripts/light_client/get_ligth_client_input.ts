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
  hexToBytes,
} from '../../../../libs/typescript/ts-utils/bls';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Config } from '../../../solidity/test/utils/constants';
import {
  computeSyncCommitteePeriodAt,
  Data,
  WitnessGeneratorInput,
} from './relayer-helper';

const ExecutionPayload = new ContainerType({
  parent_hash: new ByteVectorType(32),
  fee_recipient: new ByteVectorType(20),
  state_root: new ByteVectorType(32),
  receipts_root: new ByteVectorType(32),
  logs_bloom: new ByteVectorType(256),
  prev_randao: new ByteVectorType(32),
  block_number: new UintNumberType(8),
  gas_limit: new UintNumberType(8),
  gas_used: new UintNumberType(8),
  timestamp: new UintNumberType(8),
  extra_data: new ByteListType(32),
  base_fee_per_gas: new UintBigintType(32),
  block_hash: new ByteVectorType(32),
  transactions_root: new ByteVectorType(32),
  withdrawals_root: new ByteVectorType(32),
});

async function getBlockHeaderFromUpdate(head) {
  const { ssz } = await import('@lodestar/types');

  const blockHeader = ssz.phase0.BeaconBlockHeader.defaultValue();
  blockHeader.slot = Number.parseInt(head.slot);
  blockHeader.proposerIndex = Number.parseInt(head.proposer_index);
  blockHeader.parentRoot = hexToBytes(head.parent_root);
  blockHeader.stateRoot = hexToBytes(head.state_root);
  blockHeader.bodyRoot = hexToBytes(head.body_root);

  return blockHeader;
}

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

export async function getProofInput(
  prevUpdate: Data & { sync_committee: any; sync_committee_branch: any },
  update: Data,
  config: Config,
): Promise<WitnessGeneratorInput> {
  const { ssz } = await import('@lodestar/types');

  let syncCommitteePubkeys: PointG1[] = prevUpdate.sync_committee.pubkeys.map(
    x => PointG1.fromHex(x.slice(2)),
  );

  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(
    update.sync_aggregate.sync_committee_bits,
  );

  let signature: PointG2 = PointG2.fromSignature(
    formatHex(update.sync_aggregate.sync_committee_signature),
  );

  const prevBlockHeader = await getBlockHeaderFromUpdate(
    prevUpdate.attested_header.beacon,
  );

  const prevBlockHeaderStateRootProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['state_root'],
    prevBlockHeader,
  ).map(x => hexToBits(x));

  const prevBlockHeaderHash =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(prevBlockHeader);

  const nextBlockHeader = await getBlockHeaderFromUpdate(
    update.attested_header.beacon,
  );
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

  let syncCommitteeBranch = prevUpdate.sync_committee_branch.map(x =>
    hexToBits(x),
  );

  let finalizedHeader = await getBlockHeaderFromUpdate(
    update.finalized_header.beacon,
  );

  let finalizedHeaderHash =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedHeader);

  let finalityBranchBits = update.finality_branch.map(x => hexToBits(x));

  let finalizedHeaderBodyRootProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['body_root'],
    finalizedHeader,
  );

  let prevHeaderFinalizedBranch = prevUpdate.finality_branch.map(x =>
    hexToBits(x),
  );

  let prevFinalizedHeader = ssz.phase0.BeaconBlockHeader.fromJson(
    prevUpdate.finalized_header.beacon,
  );

  let prevFinalizedHeaderBranch = prevUpdate.finality_branch;
  let prevHeaderFinalizedSlotBranch = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['slot'],
    prevFinalizedHeader,
  ).map(x => hexToBits(x));

  let prevFinalizedHeaderStateProof = getMerkleProof(
    ssz.phase0.BeaconBlockHeader,
    ['state_root'],
    prevFinalizedHeader,
  );

  const executionPayload = ExecutionPayload.fromJson(
    update.finalized_header.execution,
  );
  const executionPayloadStateProof = getMerkleProof(
    ExecutionPayload,
    ['state_root'],
    executionPayload,
  );

  let dataView = new DataView(new ArrayBuffer(8));
  dataView.setBigUint64(
    0,
    BigInt(update.finalized_header.beacon.proposer_index),
  );
  let proposer_index = dataView.getBigUint64(0, true);
  proposer_index = BigInt(
    '0x' + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'),
  );

  return {
    prevHeaderHash: hexToBits(bytesToHex(prevBlockHeaderHash)),
    nextHeaderHash: hexToBits(bytesToHex(nextBlockHeaderHash)),
    prevFinalizedHeaderRoot: hexToBits(
      bytesToHex(
        ssz.phase0.BeaconBlockHeader.hashTreeRoot(prevFinalizedHeader),
      ),
    ),
    prevFinalizedHeaderRootBranch: [
      ...prevHeaderFinalizedBranch,
      ...prevBlockHeaderStateRootProof,
    ],
    prevHeaderFinalizedStateRoot: hexToBits(
      prevUpdate.finalized_header.beacon.state_root,
    ),
    prevHeaderFinalizedStateRootBranch: prevFinalizedHeaderStateProof.map(x =>
      hexToBits(x),
    ),

    // prevHeaderStateRoot: hexToBits(bytesToHex(prevBlockHeader.stateRoot)),
    // prevHeaderStateRootBranch: prevBlockHeaderStateRootProof,

    prevHeaderFinalizedSlot: prevFinalizedHeader.slot,
    prevHeaderFinalizedSlotBranch: [...prevHeaderFinalizedSlotBranch],
    nextHeaderSlot: nextBlockHeader.slot,
    nextHeaderSlotBranch: nextHeaderSlotBranch,

    signatureSlot: update.signature_slot,

    signatureSlotSyncCommitteePeriod: computeSyncCommitteePeriodAt(
      Number(update.signature_slot),
    ),
    finalizedHeaderSlotSyncCommitteePeriod: computeSyncCommitteePeriodAt(
      prevFinalizedHeader.slot,
    ),
    finalizedHeaderRoot: hexToBits(bytesToHex(finalizedHeaderHash)),
    finalizedHeaderBranch: [
      ...finalityBranchBits,
      ...nextBlockHeaderStateRootProof,
    ],

    execution_state_root: hexToBits(
      update.finalized_header.execution.state_root,
    ),
    execution_state_root_branch: [
      ...executionPayloadStateProof,
      ...update.finalized_header.execution_branch,
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
    aggregatedKey: hexToBits(prevUpdate.sync_committee.aggregate_pubkey, 384),
    syncCommitteeBranch: [...syncCommitteeBranch],
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
