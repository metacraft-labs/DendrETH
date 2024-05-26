import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { validatorFromValidatorJSON } from '@dendreth/relay/utils/converters';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { IndexedValidator } from '@dendreth/relay/types/types';
import config from '../../../common_config.json';
import { CommitmentMapperScheduler } from '../lib/scheduler';
import { Tree, zeroNode } from '@chainsafe/persistent-merkle-tree';
import CONSTANTS from '../../../kv_db_constants.json';
// @ts-ignore
import { getDepthByGindex, getLastSlotInEpoch, indexFromGindex, panic, range, sleep } from '@dendreth/utils/ts-utils/common-utils';
import chalk from 'chalk';
import { bitsToHex } from '@dendreth/utils/ts-utils/hex-utils';
import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import { getDummyCommitmentMapperInput } from '../../utils/common_utils';

let zeroHashes: string[] = [];

(async () => {
  const options = new CommandLineOptionsBuilder()
    .option('take', {
      type: 'number',
    })
    .build();

  // Pre-calc zero hashes
  zeroHashes = Array.from({ length: 41 }, (_, level) =>
    bytesToHex(zeroNode(level).root),
  ).reverse();

  const redis = new Redis(config['redis-host'], Number(config['redis-port']));
  const api = await getBeaconApi(config['beacon-node']);
  const eventSource = api.subscribeForEvents(['finalized_checkpoint']);

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(config);

  let lastFinalizedCheckpoint = await api.getLastFinalizedCheckpoint();
  let lastVerifiedSlot = BigInt(
    (await redis.get(CONSTANTS.lastVerifiedSlotKey))!,
  );

  eventSource.addEventListener('finalized_checkpoint', async (event: any) => {
    lastFinalizedCheckpoint = BigInt(JSON.parse(event.data).epoch);
  });

  let lastProcessedSlot = BigInt(
    (await redis.get(CONSTANTS.lastProcessedSlotKey))!,
  );
  setInterval(async () => {
    lastProcessedSlot = BigInt(
      (await redis.get(CONSTANTS.lastProcessedSlotKey))!,
    );
  }, 10000);

  while (true) {
    while (
      lastVerifiedSlot < lastProcessedSlot &&
      lastVerifiedSlot < getLastSlotInEpoch(lastFinalizedCheckpoint)
    ) {
      const verified = await verifySlot(
        api,
        redis,
        scheduler,
        lastVerifiedSlot + 1n,
        options['take'],
      );

      if (verified) {
        ++lastVerifiedSlot;
        await redis.set(
          CONSTANTS.lastVerifiedSlotKey,
          lastVerifiedSlot.toString(),
        );
      } else {
        break;
      }
    }
    await sleep(10000);
  }
})();

async function nodesAreSame(
  redis: Redis,
  newValidatorsTree: Tree,
  gindex: bigint,
  slot: bigint,
): Promise<boolean> {
  const lastChangeSlot = await redis.getSlotWithLatestChange(
    `${CONSTANTS.validatorProofKey}:${gindex}`,
    slot,
  );
  let node = await redis.get(
    `${CONSTANTS.validatorProofKey}:${gindex}:${lastChangeSlot}`,
  );

  const sha256 =
    node !== null
      ? bitsToHex(JSON.parse(node).sha256Hash)
      : zeroHashes[getDepthByGindex(Number(gindex))];

  const newNodeSha256 = bytesToHex(newValidatorsTree.getNode(gindex).root);
  return sha256 === newNodeSha256;
}

async function getValidatorsDiff(
  api: BeaconApi,
  redis: Redis,
  newBeaconState: any,
  slot: bigint,
): Promise<{
  changedValidators: IndexedValidator[];
  expectedValidatorsLength: number;
}> {
  const currentSSZFork = await api.getCurrentSSZ(slot);
  const validatorsViewDU =
    currentSSZFork.BeaconState.fields.validators.toViewDU(
      newBeaconState.validators,
    );
  const newValidatorsTree = new Tree(validatorsViewDU.node.left);

  // The roots are the same
  if (await nodesAreSame(redis, newValidatorsTree, 1n, slot)) {
    return {
      changedValidators: [],
      expectedValidatorsLength: newBeaconState.validators.length,
    };
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
          !(await nodesAreSame(redis, newValidatorsTree, childGindex, slot))
        ) {
          newChangedNodes.push(childGindex);
        }
      }
    }
    changedNodes = newChangedNodes;
  }

  const changedValidatorIndices = changedNodes.map(gindex =>
    indexFromGindex(gindex, 40n),
  );
  const changedValidators = changedValidatorIndices
    .filter(index => index < newBeaconState.validators.length)
    .map(index => ({
      index: Number(index),
      validator: newBeaconState.validators[Number(index)],
    }));

  return {
    changedValidators,
    expectedValidatorsLength: newBeaconState.validators.length,
  };
}

/// Returns true on sucessfully verified slot
async function verifySlot(
  api: BeaconApi,
  redis: Redis,
  scheduler: CommitmentMapperScheduler,
  slot: bigint,
  take: number | undefined = undefined,
): Promise<boolean> {
  console.log(
    chalk.bold.blue(`Verifying slot ${chalk.bold.cyan(slot.toString())}...`),
  );
  const currentSSZFork = await api.getCurrentSSZ(slot);

  try {
    const { beaconState } =
      (await api.getBeaconState(slot)) ||
      panic('Could not fetch beacon state!');
    beaconState.validators = beaconState.validators.slice(0, take);
    const validatorsRoot = bytesToHex(
      currentSSZFork.BeaconState.fields.validators.hashTreeRoot(
        beaconState.validators,
      ),
    );

    let storedValidatorsRoot: String | null = null;
    while (storedValidatorsRoot === null) {
      const latestValidatorChangeSlot = await redis.getSlotWithLatestChange(
        `${CONSTANTS.validatorProofKey}:1`,
        BigInt(slot),
      );
      if (latestValidatorChangeSlot !== null) {
        storedValidatorsRoot = await redis.getValidatorsRoot(
          latestValidatorChangeSlot,
        );
      }
      await sleep(1000);
    }

    if (validatorsRoot !== storedValidatorsRoot) {
      console.log(
        chalk.bold.red(
          `Validators roots for slot ${slot} differ: expected "${validatorsRoot}", got "${storedValidatorsRoot}. Rescheduling tasks..."`,
        ),
      );
      // reschedule tasks for slot
      let { changedValidators, expectedValidatorsLength } =
        await getValidatorsDiff(api, redis, beaconState, slot);

      const actualValidatorsLen =
        (await redis.getValidatorsLengthForSlot(slot)) ||
        panic(`Could not fetch validators length for slot ${slot}`);
      if (expectedValidatorsLength < actualValidatorsLen) {
        // There are actually less validators in the current state than we know
        // about, so we zero out the obsolete ones
        const validatorsToBeZeroesIndices = range(
          expectedValidatorsLength,
          actualValidatorsLen,
        );

        for (const index of validatorsToBeZeroesIndices) {
          await scheduler.scheduleZeroOutValidatorTask(index, slot);
        }

        const zeroValidators = validatorsToBeZeroesIndices.map(index => ({
          index,
          validator: validatorFromValidatorJSON(
            getDummyCommitmentMapperInput().validator,
          ),
        }));
        changedValidators = changedValidators.concat(zeroValidators);
      }

      await scheduler.saveValidatorsInBatches(changedValidators, slot);
      await redis.deleteValidatorsRoot(slot);
      await redis.setValidatorsLength(slot, beaconState.validators.length);
      return false;
    }
  } catch (error) {
    console.error(error);
    return false;
  }

  return true;
}
