import { BitVectorType } from '@chainsafe/ssz';
import { PointG1, PointG2 } from '@noble/bls12-381';
import { writeFileSync } from 'fs';
import * as path from 'path';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
} from '../../../../libs/typescript/ts-utils/bls';
import { getFilesInDir } from '../../../../libs/typescript/ts-utils/data';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import * as constants from '../../../solidity/test/utils/constants';

(async () => {
  const { ssz } = await import('@lodestar/types');

  function getBlockHeaderFromUpdate(head) {
    const blockHeader = ssz.phase0.BeaconBlockHeader.defaultValue();
    blockHeader.slot = Number.parseInt(head.slot);
    blockHeader.proposerIndex = Number.parseInt(head.proposer_index);
    blockHeader.parentRoot = hexToBytes(head.parent_root);
    blockHeader.stateRoot = hexToBytes(head.state_root);
    blockHeader.bodyRoot = hexToBytes(head.body_root);

    return blockHeader;
  }

  async function getProof(prevUpdate, update, finalizedExecutionPayload) {
    let points: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x =>
      PointG1.fromHex(x.slice(2)),
    );

    const SyncCommitteeBits = new BitVectorType(512);
    let bitmask = SyncCommitteeBits.fromJson(
      update.sync_aggregate.sync_committee_bits,
    );

    let signature: PointG2 = PointG2.fromSignature(
      formatHex(update.sync_aggregate.sync_committee_signature),
    );

    const prevBlockHeader = getBlockHeaderFromUpdate(
      prevUpdate.attested_header.beacon,
    );

    const prevBlockHeaderView =
      ssz.phase0.BeaconBlockHeader.toViewDU(prevBlockHeader);
    const prevBlockHeaderTree = new Tree(prevBlockHeaderView.node);

    console.log(
      ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
    );

    const prevBlockHeaderStateRootProof = prevBlockHeaderTree
      .getSingleProof(
        ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
      )
      .map(bytesToHex)
      .map(x =>
        BigInt('0x' + x)
          .toString(2)
          .padStart(256, '0')
          .split(''),
      );

    console.log('state proof length: ', prevBlockHeaderStateRootProof.length);

    console.log(
      BigInt('0b' + 55n.toString(2) + 11n.toString(2)),
    );

    const prevBlockHeaderHash =
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(prevBlockHeader);

    const nextBlockHeader = getBlockHeaderFromUpdate(
      update.attested_header.beacon,
    );
    const nextBlockHeaderHash =
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(nextBlockHeader);

    const nextBlockHeaderView =
      ssz.phase0.BeaconBlockHeader.toViewDU(nextBlockHeader);
    const nextBlockHeaderTree = new Tree(nextBlockHeaderView.node);

    const nextBlockHeaderStateRootProof = nextBlockHeaderTree
      .getSingleProof(
        ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
      )
      .map(bytesToHex)
      .map(x =>
        BigInt('0x' + x)
          .toString(2)
          .padStart(256, '0')
          .split(''),
      );

    let nextSyncCommitteeBranch = prevUpdate.next_sync_committee_branch.map(x =>
      BigInt(x).toString(2).padStart(256, '0').split(''),
    );

    let finalizedHeader = getBlockHeaderFromUpdate(
      update.finalized_header.beacon,
    );

    const finalizedHeaderVew =
      ssz.phase0.BeaconBlockHeader.toViewDU(finalizedHeader);
    const finalizedHeaderTree = new Tree(finalizedHeaderVew.node);

    const finalizedHeaderBodyRootProof = finalizedHeaderTree
      .getSingleProof(
        ssz.phase0.BeaconBlockHeader.getPathInfo(['body_root']).gindex,
      )
      .map(bytesToHex)
      .map(x =>
        BigInt('0x' + x)
          .toString(2)
          .padStart(256, '0')
          .split(''),
      );

    let finalizedHeaderHash =
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedHeader);

    console.log(bytesToHex(finalizedHeaderHash));

    let finalityBranch = update.finality_branch.map(x =>
      BigInt(x).toString(2).padStart(256, '0').split(''),
    );

    let executionStateRoot = finalizedExecutionPayload.state_root;
    let executionPayload = ssz.capella.ExecutionPayload.fromJson(
      finalizedExecutionPayload,
    );
    let executionPayloadView =
      ssz.capella.ExecutionPayload.toViewDU(executionPayload);
    let executionPayloadTree = new Tree(executionPayloadView.node);
    const proof = executionPayloadTree.getSingleProof(
      ssz.capella.ExecutionPayload.getPathInfo(['state_root']).gindex,
    );

    const executionBranch = [
      ...proof.map(bytesToHex),
      ...update.finalized_header.execution_branch,
    ].map(x =>
      BigInt('0x' + formatHex(x))
        .toString(2)
        .padStart(256, '0')
        .split(''),
    );

    let dataView = new DataView(new ArrayBuffer(8));
    dataView.setBigUint64(0, BigInt(prevBlockHeader.slot));
    let slot = dataView.getBigUint64(0, true);
    slot = BigInt('0x' + slot.toString(16).padStart(16, '0').padEnd(64, '0'));

    dataView.setBigUint64(0, BigInt(prevBlockHeader.proposerIndex));
    let proposer_index = dataView.getBigUint64(0, true);
    proposer_index = BigInt(
      '0x' + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'),
    );

    return {
      points: points.map(x => [
        bigint_to_array(55, 7, x.toAffine()[0].value),
        bigint_to_array(55, 7, x.toAffine()[1].value),
      ]),
      prevHeaderHash: BigInt('0x' + bytesToHex(prevBlockHeaderHash))
        .toString(2)
        .padStart(256, '0')
        .split(''),
      nextHeaderHash: BigInt('0x' + bytesToHex(nextBlockHeaderHash))
        .toString(2)
        .padStart(256, '0')
        .split(''),
      slot: slot.toString(2).padStart(256, '0').split(''),
      proposer_index: proposer_index.toString(2).padStart(256, '0').split(''),
      parent_root: BigInt('0x' + bytesToHex(prevBlockHeader.parentRoot))
        .toString(2)
        .padStart(256, '0')
        .split(''),
      state_root: BigInt('0x' + bytesToHex(prevBlockHeader.stateRoot))
        .toString(2)
        .padStart(256, '0')
        .split(''),
      body_root: BigInt('0x' + bytesToHex(prevBlockHeader.bodyRoot))
        .toString(2)
        .padStart(256, '0')
        .split(''),

      finalized_header_root: BigInt('0x' + bytesToHex(finalizedHeaderHash))
        .toString(2)
        .padStart(256, '0')
        .split(''),

      finalized_branch: [...finalityBranch, ...nextBlockHeaderStateRootProof],
      execution_state_root: BigInt(executionStateRoot)
        .toString(2)
        .padStart(256, '0')
        .split(''),
      execution_state_root_branch: [
        ...executionBranch,
        ...finalizedHeaderBodyRootProof,
      ],
      fork_version: BigInt('0x' + bytesToHex(constants.CAPELLA_FORK_VERSION))
        .toString(2)
        .padStart(32, '0')
        .split(''),
      GENESIS_VALIDATORS_ROOT: BigInt(
        '0x' + bytesToHex(constants.GENESIS_FORK_VERSION),
      )
        .toString(2)
        .padStart(256, '0')
        .split(''),
      DOMAIN_SYNC_COMMITTEE: BigInt(
        '0x' + bytesToHex(constants.DOMAIN_SYNC_COMMITTEE),
      )
        .toString(2)
        .padStart(32, '0')
        .split(''),
      aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey)
        .toString(2)
        .padStart(384, '0')
        .split(''),
      syncCommiteeBranch: [
        ...nextSyncCommitteeBranch,
        ...prevBlockHeaderStateRootProof,
      ],
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

  const UPDATES = getFilesInDir(path.join(__dirname, 'updates'));

  let prevUpdate = JSON.parse(UPDATES[0].toString()).data;

  const finalized_execution_payload = {
    parent_hash:
      '0x762d40e0a4e1ea1e5c21852f3407edf8b7fafab693bdddb70edafa657efab6ca',
    fee_recipient: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
    state_root:
      '0x48c68defe313e6cc47cdebe39d8bc76c4dac38797fb74153db10af91be92419e',
    receipts_root:
      '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
    logs_bloom:
      '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
    prev_randao:
      '0x790180e7f120c1be1751f6e3548803b595f30cbf922ae54e87bedf59c68a489c',
    block_number: '56957',
    gas_limit: '30000000',
    gas_used: '0',
    timestamp: '1675961712',
    extra_data: '0x',
    base_fee_per_gas: '7',
    block_hash:
      '0x122c3e5d954d9c101fbdcd3288156db84e8afb0046144ff256ec3510f020fba4',
    transactions: [],
    withdrawals: [
      {
        index: '228854',
        validator_index: '48256',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228855',
        validator_index: '48257',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228856',
        validator_index: '48258',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228857',
        validator_index: '48259',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228858',
        validator_index: '48260',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228859',
        validator_index: '48261',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228860',
        validator_index: '48262',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228861',
        validator_index: '48263',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228862',
        validator_index: '48264',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228863',
        validator_index: '48265',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228864',
        validator_index: '48266',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228865',
        validator_index: '48267',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '181970',
      },
      {
        index: '228866',
        validator_index: '48268',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228867',
        validator_index: '48269',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
      {
        index: '228868',
        validator_index: '48270',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '181970',
      },
      {
        index: '228869',
        validator_index: '48271',
        address: '0xf97e180c050e5ab072211ad2c213eb5aee4df134',
        amount: '191601',
      },
    ],
  };

  for (let update of UPDATES.slice(1, 2)) {
    writeFileSync(
      path.join(__dirname, 'input.json'),
      JSON.stringify(
        await getProof(
          prevUpdate,
          JSON.parse(update.toString()).data,
          finalized_execution_payload,
        ),
      ),
    );
  }
})();
