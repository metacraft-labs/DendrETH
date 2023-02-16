import { Worker, Queue } from 'bullmq';
import { exec as _exec } from 'child_process';
import { readFile, writeFile } from 'fs/promises';
import path from 'path';
import {
  computeSyncCommitteePeriodAt,
  ProofInputType,
  PROOF_GENERATOR_QUEUE,
  RELAYER_INPUTS_FOLDER,
  RELAYER_UPDATES_FOLDER,
  State,
  Update,
  UPDATE_POLING_QUEUE,
} from '../relayer-helper';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { getProofInput } from '../get_ligth_client_input';
import { ZHEAJIANG_TESNET } from '../../../../solidity/test/utils/constants';
import { bytesToHex } from '../../../../../libs/typescript/ts-utils/bls';
import * as config from '../config.json';

const proofGenertorQueue = new Queue<ProofInputType>(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

new Worker<void>(
  UPDATE_POLING_QUEUE,
  async () => {
    const state = await getState();

    const update = await getUpdate({ ...state });

    if (!update) return;

    const prevUpdate: Update = JSON.parse(
      await readFile(
        path.join(
          __dirname,
          '..',
          RELAYER_UPDATES_FOLDER,
          `update_${state.lastDownloadedUpdate}.json`,
        ),
        'utf8',
      ),
    );

    await extendPrevUpdateWithSyncCommittee(
      prevUpdate,
      Number(update.data.signature_slot),
    );

    const proofInput = await getProofInput(
      prevUpdate.data as any,
      update.data,
      ZHEAJIANG_TESNET,
    );

    await writeFile(
      path.join(
        __dirname,
        '..',
        RELAYER_INPUTS_FOLDER,
        `input_${prevUpdate.data.attested_header.beacon.slot}_${update.data.attested_header.beacon.slot}.json`,
      ),
      JSON.stringify(proofInput),
    );

    proofGenertorQueue.add('proofGenerate', {
      prevUpdateSlot: Number(prevUpdate.data.attested_header.beacon.slot),
      updateSlot: Number(update.data.attested_header.beacon.slot),
      proofInput: proofInput,
    });
  },
  {
    connection: {
      host: config.redisHost,
      port: config.redisPort,
    },
  },
);

async function extendPrevUpdateWithSyncCommittee(
  prevUpdate: Update,
  signature_slot: number,
) {
  const beaconStateSZZ = await fetch(
    `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v2/debug/beacon/states/${prevUpdate.data.attested_header.beacon.slot}`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const { ssz } = await import('@lodestar/types');
  const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);
  const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  const tree = new Tree(beaconStateView.node);

  const prevUpdateFinalizedSyncCommmitteePeriod = computeSyncCommitteePeriodAt(
    Number(prevUpdate.data.finalized_header.beacon.slot),
  );
  const currentSyncCommitteePeriod =
    computeSyncCommitteePeriodAt(signature_slot);

  const sync_committee_branch = tree
    .getSingleProof(
      ssz.capella.BeaconState.getPathInfo([
        prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
          ? 'current_sync_committee'
          : 'next_sync_committee',
      ]).gindex,
    )
    .map(x => '0x' + bytesToHex(x));

  prevUpdate.data['sync_committee_branch'] = sync_committee_branch;
  prevUpdate.data['sync_committee'] = {
    pubkeys: beaconState[
      prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
        ? 'currentSyncCommittee'
        : 'nextSyncCommittee'
    ].pubkeys.map(x => '0x' + bytesToHex(x)),
    aggregate_pubkey:
      '0x' +
      bytesToHex(
        beaconState[
          prevUpdateFinalizedSyncCommmitteePeriod === currentSyncCommitteePeriod
            ? 'currentSyncCommittee'
            : 'nextSyncCommittee'
        ].aggregatePubkey,
      ),
  };
}

async function getUpdate(state: State): Promise<Update | undefined> {
  const update = await (
    await fetch(
      `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v1/beacon/light_client/finality_update`,
    )
  )
    .json()
    .catch(e => console.log(e));

  const updateSlot = Number(update.data.attested_header.beacon.slot);

  if (state.lastDownloadedUpdate >= updateSlot) {
    console.log('There is no new header to update');
    return undefined;
  }

  state.lastDownloadedUpdate = updateSlot;
  await writeFile(
    path.join(__dirname, '..', 'state.json'),
    JSON.stringify(state),
  );

  await writeFile(
    path.join(
      __dirname,
      '..',
      RELAYER_UPDATES_FOLDER,
      `update_${updateSlot}.json`,
    ),
    JSON.stringify(update),
  );

  return update;
}

async function getState(): Promise<State> {
  return JSON.parse(
    await readFile(path.join(__dirname, '..', 'state.json'), 'utf-8'),
  );
}
