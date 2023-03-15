import { computeSyncCommitteePeriodAt } from './relayer-helper';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';
import { ZHEAJIANG_TESNET } from '../../../solidity/test/utils/constants';
import { getProofInput } from './get_ligth_client_input';

export async function getInputFromTo(
  from: number,
  to: number,
  config: { beaconRestApiHost: string; beaconRestApiPort: number },
) {
  const { ssz } = await import('@lodestar/types');

  // get prevHeader
  const prevBlockHeaderResult = await (
    await fetch(
      `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v1/beacon/headers/${from}`,
    )
  ).json();

  console.log('prevHeaderResult:', prevBlockHeaderResult);

  const prevBlockHeader = ssz.phase0.BeaconBlockHeader.fromJson(
    prevBlockHeaderResult.data.header.message,
  );

  let nextBlockSlot = to;

  let nextBlockHeaderResult;
  while (true) {
    nextBlockHeaderResult = await (
      await fetch(
        `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v1/beacon/headers/${nextBlockSlot}`,
      )
    ).json();

    if (nextBlockHeaderResult.code !== 404) {
      break;
    }

    nextBlockSlot++;
  }

  console.log('nextBlockHeaderResult:', nextBlockHeaderResult);

  const nextBlockHeader = ssz.phase0.BeaconBlockHeader.fromJson(
    nextBlockHeaderResult.data.header.message,
  );

  const prevBeaconStateSZZ = await fetch(
    `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/debug/beacon/states/${from}`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const prevBeaconSate =
    ssz.capella.BeaconState.deserialize(prevBeaconStateSZZ);
  const prevBeaconStateView = ssz.capella.BeaconState.toViewDU(prevBeaconSate);
  const prevStateTree = new Tree(prevBeaconStateView.node);

  const prevFinalizedHeaderResult = await (
    await fetch(
      `http://${config.beaconRestApiHost}:${
        config.beaconRestApiPort
      }/eth/v1/beacon/headers/${
        '0x' + bytesToHex(prevBeaconSate.finalizedCheckpoint.root)
      }`,
    )
  ).json();

  console.log('prevUpdateFinalizedHeaderResult', prevFinalizedHeaderResult);

  const prevFinalizedHeader = ssz.phase0.BeaconBlockHeader.fromJson(
    prevFinalizedHeaderResult.data.header.message,
  );

  const prevUpdateFinalizedSyncCommmitteePeriod = computeSyncCommitteePeriodAt(
    prevFinalizedHeader.slot,
  );
  const currentSyncCommitteePeriod = computeSyncCommitteePeriodAt(
    nextBlockHeader.slot,
  );

  const prevFinalizedHeaderBeaconStateSZZ = await fetch(
    `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/debug/beacon/states/${prevFinalizedHeader.slot}`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const prevFinalizedBeaconState = ssz.capella.BeaconState.deserialize(
    prevFinalizedHeaderBeaconStateSZZ,
  );
  const prevFinalizedBeaconStateView = ssz.capella.BeaconState.toViewDU(
    prevFinalizedBeaconState,
  );
  const prevFinalizedBeaconStateTree = new Tree(
    prevFinalizedBeaconStateView.node,
  );

  const syncCommitteeBranch = prevFinalizedBeaconStateTree
    .getSingleProof(
      ssz.capella.BeaconState.getPathInfo([
        prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
          ? 'current_sync_committee'
          : 'next_sync_committee',
      ]).gindex,
    )
    .map(x => '0x' + bytesToHex(x));

  const syncCommittee = {
    pubkeys: prevFinalizedBeaconState[
      prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
        ? 'currentSyncCommittee'
        : 'nextSyncCommittee'
    ].pubkeys.map(x => '0x' + bytesToHex(x)),
    aggregate_pubkey:
      '0x' +
      bytesToHex(
        prevFinalizedBeaconState[
          prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
            ? 'currentSyncCommittee'
            : 'nextSyncCommittee'
        ].aggregatePubkey,
      ),
  };

  const prevFinalityBranch = prevStateTree
    .getSingleProof(
      ssz.capella.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
        .gindex,
    )
    .map(x => '0x' + bytesToHex(x));

  const nextBeaconStateSZZ = await fetch(
    `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/debug/beacon/states/${nextBlockSlot}`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const nextBeaconSate =
    ssz.capella.BeaconState.deserialize(nextBeaconStateSZZ);
  const nextBeaconStateView = ssz.capella.BeaconState.toViewDU(nextBeaconSate);
  const nextStateTree = new Tree(nextBeaconStateView.node);

  const nextFinalizedHeaderResult = await (
    await fetch(
      `http://${config.beaconRestApiHost}:${
        config.beaconRestApiPort
      }/eth/v1/beacon/headers/${
        '0x' + bytesToHex(nextBeaconSate.finalizedCheckpoint.root)
      }`,
    )
  ).json();

  console.log('nextFinalizedHeaderResult', nextFinalizedHeaderResult);

  const finalizedHeader = ssz.phase0.BeaconBlockHeader.fromJson(
    nextFinalizedHeaderResult.data.header.message,
  );

  const finalityBranch = nextStateTree
    .getSingleProof(
      ssz.capella.BeaconState.getPathInfo(['finalized_checkpoint', 'root'])
        .gindex,
    )
    .map(x => '0x' + bytesToHex(x));

  let signature_slot = nextBlockSlot + 1;
  let blockHeaderBodyResult;

  while (true) {
    blockHeaderBodyResult = await (
      await fetch(
        `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/beacon/blocks/${signature_slot}`,
      )
    ).json();

    if (blockHeaderBodyResult.code !== 404) {
      break;
    }

    signature_slot++;
  }

  console.log('blockHeaderBodyResult', blockHeaderBodyResult);

  const sync_aggregate = blockHeaderBodyResult.data.message.body.sync_aggregate;

  const finalizedBlockBodyResult = await (
    await fetch(
      `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/beacon/blocks/${finalizedHeader.slot}`,
    )
  ).json();

  const finalizedBlockBody = ssz.capella.BeaconBlockBody.fromJson(
    finalizedBlockBodyResult.data.message.body,
  );

  const finalizedBlockBodyView =
    ssz.capella.BeaconBlockBody.toViewDU(finalizedBlockBody);
  const finalizedBlockBodyTree = new Tree(finalizedBlockBodyView.node);

  const finalizedHeaderExecutionBranch = finalizedBlockBodyTree
    .getSingleProof(
      ssz.capella.BeaconBlockBody.getPathInfo(['execution_payload']).gindex,
    )
    .map(x => '0x' + bytesToHex(x));

  const finalizedBeaconStateSSZ = await fetch(
    `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/debug/beacon/states/${finalizedHeader.slot}`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const finalizedBeaconState = ssz.capella.BeaconState.deserialize(
    finalizedBeaconStateSSZ,
  );

  return {
    proofInput: await getProofInput({
      prevBlockHeader,
      nextBlockHeader,
      prevFinalizedHeader,
      syncCommitteeBranch,
      syncCommittee,
      config: ZHEAJIANG_TESNET,
      prevFinalityBranch,
      signature_slot: signature_slot,
      finalizedHeader,
      finalityBranch,
      executionPayload: finalizedBeaconState.latestExecutionPayloadHeader,
      finalizedHeaderExecutionBranch,
      sync_aggregate,
    }),
    prevUpdateSlot: prevBlockHeader.slot,
    updateSlot: nextBlockHeader.slot,
  };
}
