import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import {
  BeaconApi,
  getBeaconApi,
} from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { IndexedValidator, Validator } from '../../../relay/types/types';
import config from '../common_config.json';
import { CommitmentMapperScheduler } from './scheduler';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import CONSTANTS from '../constants/validator_commitment_constants.json';
import { panic } from '../../../libs/typescript/ts-utils/common-utils';

async function getFirstNonMissingSlotInEpoch(
  api: BeaconApi,
  epoch: number,
): Promise<number> {
  for (let relativeSlot = 0; relativeSlot < 31; ++relativeSlot) {
    const slot = epoch * 32 + relativeSlot;
    try {
      const status = await api.pingEndpoint(
        `/eth/v1/beacon/states/${slot}/root`,
      );
      if (status === 200) {
        return slot;
      }

      // const validators = await api.getBlockRootBySlot(epoch * 32 + slot);
      // return epoch * 32 + slot;
    } catch (error) {
      console.error(error);
    }
  }
  throw new Error('Did not find non-empty slot in epoch');
}

(async () => {
  const { ssz } = await import('@lodestar/types');

  const redis = new Redis(config['redis-host'], Number(config['redis-port']));
  const api = await getBeaconApi([config['beacon-node']]);
  const eventSource = await api.subscribeForEvents(['finalized_checkpoint']);

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(config);

  eventSource.addEventListener('finalized_checkpoint', async (event: any) => {
    const json = JSON.parse(event.data);
    const epoch = Number(json.epoch);

    try {
      const slot = await getFirstNonMissingSlotInEpoch(api, epoch);
      const { beaconState } =
        (await api.getBeaconState(slot)) ||
        panic('Could not fetch beacon state');
      const validatorsRoot = bytesToHex(
        ssz.capella.BeaconState.fields.validators.hashTreeRoot(
          beaconState.validators,
        ),
      );

      let storedValidatorsRoot: String | null = null;
      while (storedValidatorsRoot !== null) {
        const latestValidatorsChangedEpoch = await redis.getLatestEpoch(
          `${CONSTANTS.validatorProofKey}:0`,
          BigInt(epoch),
        );
        if (latestValidatorsChangedEpoch !== null) {
          storedValidatorsRoot = await redis.getValidatorsRoot(
            latestValidatorsChangedEpoch,
          );
        }
      }

      if (validatorsRoot !== storedValidatorsRoot) {
        console.log(
          `Validators roots for epoch ${epoch} differ: expected "${validatorsRoot}", got "${storedValidatorsRoot}"`,
        );
        // reschedule tasks for epoch
        const changedValidators = await getValidatorsDiff(
          redis,
          beaconState,
          BigInt(epoch),
        );
        await scheduler.saveValidatorsInBatches(
          changedValidators,
          BigInt(epoch),
        );
        await scheduler.setValidatorsLength(
          BigInt(epoch),
          beaconState.validators.length,
        );
      }
    } catch (error) {
      console.error(error);
    }

    console.log(event);
  });
})();

async function nodesAreSame(
  redis: Redis,
  newValidatorsTree: Tree,
  gindex: bigint,
  epoch: bigint,
): Promise<boolean> {
  const redisGindex = gindex - 1n; // TODO: Delete this and use gindex when we change the indexing scheme

  const lastChangeEpoch = redis.getLatestEpoch(
    `${CONSTANTS.validatorProofKey}:${redisGindex}`,
    epoch,
  );
  const node = await redis.get(
    `${CONSTANTS.validatorProofKey}:${lastChangeEpoch}`,
  );
  const sha256 = bytesToHex(bitArrayToByteArray(JSON.parse(node!).sha256Hash));
  const newNodeSha256 = bytesToHex(newValidatorsTree.getNode(gindex).root);
  return sha256 === newNodeSha256;
}

async function getValidatorsDiff(
  redis: Redis,
  newBeaconState: any,
  epoch: bigint,
): Promise<IndexedValidator[]> {
  const { ssz } = await import('@lodestar/types');

  const validatorsViewDU = ssz.capella.BeaconState.fields.validators.toViewDU(
    newBeaconState.validators,
  );
  const newValidatorsTree = new Tree(validatorsViewDU.node);

  // The roots are the same
  if (await nodesAreSame(redis, newValidatorsTree, 1n, epoch)) {
    return [];
  }

  let changedNodes = [1n];

  for (let depth = 0; depth < 40; ++depth) {
    const newChangedNodes: bigint[] = [];
    for (const changedNodeGindex of changedNodes) {
      // fetch the node at gindex from redis
      for (const childGindex of [
        2n * changedNodeGindex,
        2n * changedNodeGindex + 1n,
      ]) {
        if (
          !(await nodesAreSame(redis, newValidatorsTree, childGindex, epoch))
        ) {
          newChangedNodes.push(childGindex);
        }
      }

      // compare it to what we fetched from the api
      // if it's different, push gindex * 2...gindex*2+1
    }
    changedNodes = newChangedNodes;
  }

  // changedNodes now contains all the changed leaf nodes' indices

  return [];
}

function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += 2 ** (7 - bit) * hash[byte * 8 + bit];
    }
    result[byte] = value;
  }
  return result;
}

// we need a function that returns validators by an array of indices
