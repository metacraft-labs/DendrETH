import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { IndexedValidator } from '@dendreth/relay/types/types';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import config from '../common_config.json';
import { CommitmentMapperScheduler } from './scheduler';
import { Tree, zeroNode } from '@chainsafe/persistent-merkle-tree';
import CONSTANTS from '../constants/validator_commitment_constants.json';
// @ts-ignore
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import { getDepthByGindex, indexFromGindex } from './utils';
import { CommandLineOptionsBuilder } from '../cmdline';
import chalk from 'chalk';

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
  let lastVerifiedEpoch = BigInt(
    (await redis.get(CONSTANTS.lastFinalizedEpochLookupKey))!,
  );

  eventSource.addEventListener('finalized_checkpoint', async (event: any) => {
    lastFinalizedCheckpoint = BigInt(JSON.parse(event.data).epoch);
  });

  let lastProcessedEpoch = BigInt(
    (await redis.get(CONSTANTS.lastProcessedEpochLookupKey))!,
  );
  setInterval(async () => {
    lastProcessedEpoch = BigInt(
      (await redis.get(CONSTANTS.lastProcessedEpochLookupKey))!,
    );
  }, 10000);

  while (true) {
    while (
      lastVerifiedEpoch < lastProcessedEpoch &&
      lastVerifiedEpoch < lastFinalizedCheckpoint
    ) {
      ++lastVerifiedEpoch;
      const verified = await verifyEpoch(
        api,
        redis,
        scheduler,
        lastVerifiedEpoch + 1n,
        options['take'],
      );
      if (verified) {
        ++lastVerifiedEpoch;
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
  epoch: bigint,
): Promise<boolean> {
  const lastChangeEpoch = await redis.getLatestEpoch(
    `${CONSTANTS.validatorProofKey}:${gindex}`,
    epoch,
  );
  let node = await redis.get(
    `${CONSTANTS.validatorProofKey}:${gindex}:${lastChangeEpoch}`,
  );

  const sha256 =
    node !== null
      ? bytesToHex(bitArrayToByteArray(JSON.parse(node).sha256Hash))
      : zeroHashes[getDepthByGindex(Number(gindex))];

  const newNodeSha256 = bytesToHex(newValidatorsTree.getNode(gindex).root);
  return sha256 === newNodeSha256;
}

async function getValidatorsDiff(
  api: BeaconApi,
  redis: Redis,
  newBeaconState: any,
  epoch: bigint,
): Promise<IndexedValidator[]> {
  const currentSSZFork = await api.getCurrentSSZ(epoch * 32n);
  const validatorsViewDU =
    currentSSZFork.BeaconState.fields.validators.toViewDU(
      newBeaconState.validators,
    );
  const newValidatorsTree = new Tree(validatorsViewDU.node.left);

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
    }
    changedNodes = newChangedNodes;
  }

  const changedValidatorIndices = changedNodes.map(gindex =>
    indexFromGindex(gindex, 40n),
  );
  return changedValidatorIndices.map(index => ({
    index: Number(index),
    validator: newBeaconState.validators[Number(index)],
  }));
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

/// Returns true on sucessfully verified epoch
async function verifyEpoch(
  api: BeaconApi,
  redis: Redis,
  scheduler: CommitmentMapperScheduler,
  epoch: bigint,
  take: number | undefined = undefined,
): Promise<boolean> {
  console.log(
    chalk.bold.blue(`Verifying epoch: ${chalk.bold.cyan(epoch.toString())}`),
  );
  const currentSSZFork = await api.getCurrentSSZ(epoch * 32n);

  try {
    const slot = await api.getFirstNonMissingSlotInEpoch(epoch);
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
      const latestValidatorsChangedEpoch = await redis.getLatestEpoch(
        `${CONSTANTS.validatorProofKey}:1`,
        BigInt(epoch),
      );
      if (latestValidatorsChangedEpoch !== null) {
        storedValidatorsRoot = await redis.getValidatorsRoot(
          latestValidatorsChangedEpoch,
        );
      }
      await sleep(1000);
    }

    if (validatorsRoot !== storedValidatorsRoot) {
      console.log(
        chalk.bold.red(
          `Validators roots for epoch ${epoch} differ: expected "${validatorsRoot}", got "${storedValidatorsRoot}"`,
        ),
      );
      // reschedule tasks for epoch
      await redis.updateCommitmentMapperSlot(epoch, BigInt(slot));
      const changedValidators = await getValidatorsDiff(
        api,
        redis,
        beaconState,
        BigInt(epoch),
      );
      await scheduler.saveValidatorsInBatches(changedValidators, BigInt(epoch));
      await redis.setValidatorsLength(
        BigInt(epoch),
        beaconState.validators.length,
      );
      await redis.set(
        `${CONSTANTS.validatorsRootKey}:${epoch}`,
        validatorsRoot,
      );
    }

    await redis.set(CONSTANTS.lastFinalizedEpochLookupKey, epoch.toString());
  } catch (error) {
    console.error(error);
    return false;
  }
  return true;
}
