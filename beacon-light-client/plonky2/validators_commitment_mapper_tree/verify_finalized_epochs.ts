import { bytesToHex } from "../../../libs/typescript/ts-utils/bls";
import { BeaconApi, getBeaconApi } from "../../../relay/implementations/beacon-api";
import { Redis } from "@dendreth/relay/implementations/redis";
import { IndexedValidator } from "../../../relay/types/types";
import config from "../common_config.json";
import { CommitmentMapperScheduler } from "./scheduler";
import { Tree, zeroNode } from '@chainsafe/persistent-merkle-tree';
import CONSTANTS from '../constants/validator_commitment_constants.json';
// @ts-ignore
import { sleep } from '@dendreth/utils/ts-utils/common-utils";
import yargs from "yargs";
import { getDepthByGindex } from "./utils";


let zeroHashes: string[] = [];

(async () => {
  const options = yargs.option('take', {
    type: 'number',
  }).argv;

  // Pre-calc zero hashes
  zeroHashes = Array.from({ length: 41 }, (_, level) => bytesToHex(zeroNode(level).root)).reverse();

  const redis = new Redis(config['redis-host'], Number(config['redis-port']));
  const api = await getBeaconApi([config['beacon-node']]);
  const eventSource = await api.subscribeForEvents(['finalized_checkpoint']);

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(config);

  let lastFinalizedCheckpoint = await api.getLastFinalizedCheckpoint();
  let lastVerifiedEpoch = BigInt((await redis.get(CONSTANTS.lastFinalizedEpochLookupKey))!);

  eventSource.on('finalized_checkpoint', async (event: any) => {
    lastFinalizedCheckpoint = BigInt(JSON.parse(event.data).epoch);
  });

  let lastProcessedEpoch = BigInt((await redis.get(CONSTANTS.lastProcessedEpochLookupKey))!);
  setInterval(async () => {
    lastProcessedEpoch = BigInt((await redis.get(CONSTANTS.lastProcessedEpochLookupKey))!);
  }, 10000);

  while (true) {
    while (lastVerifiedEpoch < lastProcessedEpoch && lastVerifiedEpoch < lastFinalizedCheckpoint) {
      ++lastVerifiedEpoch;
      await verifyEpoch(api, redis, scheduler, lastVerifiedEpoch, options['take']);
    }
    await sleep(1000);
  }
})();

async function nodesAreSame(
  redis: Redis,
  newValidatorsTree: Tree,
  gindex: bigint,
  epoch: bigint,
): Promise<boolean> {
  const redisGindex = gindex - 1n; // TODO: Delete this and use gindex when we change the indexing scheme

  const lastChangeEpoch = await redis.getLatestEpoch(`${CONSTANTS.validatorProofKey}:${redisGindex}`, epoch);
  let node = await redis.get(`${CONSTANTS.validatorProofKey}:${redisGindex}:${lastChangeEpoch}`);

  const sha256 = (node !== null)
    ? bytesToHex(bitArrayToByteArray(JSON.parse(node).sha256Hash))
    : zeroHashes[getDepthByGindex(Number(redisGindex))];

  const newNodeSha256 = bytesToHex(newValidatorsTree.getNode(gindex).root);
  return sha256 === newNodeSha256;
}

async function getValidatorsDiff(
  redis: Redis,
  newBeaconState: any,
  epoch: bigint,
): Promise<IndexedValidator[]> {
  const { ssz } = await import('@lodestar/types');
  const validatorsViewDU = ssz.capella.BeaconState.fields.validators.toViewDU(newBeaconState.validators);
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

  const changedValidatorIndices = changedNodes.map(gindex => gindex - 2n ** 40n);
  return changedValidatorIndices.map(index => (
    {
      index: Number(index),
      validator: newBeaconState.validators[Number(index)]
    }
  ));
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

async function verifyEpoch(api: BeaconApi, redis: Redis, scheduler: CommitmentMapperScheduler, epoch: bigint, take: number | undefined = undefined) {
  console.log(`Verifying epoch: ${epoch}`)
  const { ssz } = await import('@lodestar/types');
  try {
    const slot = await getFirstNonMissingSlotInEpoch(api, Number(epoch));
    const { beaconState } = await api.getBeaconState(slot);
    beaconState.validators = beaconState.validators.slice(0, take);
    const validatorsRoot = bytesToHex(ssz.capella.BeaconState.fields.validators.hashTreeRoot(beaconState.validators));

    let storedValidatorsRoot: String | null = null;
    while (storedValidatorsRoot === null) {
      const latestValidatorsChangedEpoch = await redis.getLatestEpoch(`${CONSTANTS.validatorProofKey}:0`, BigInt(epoch));
      if (latestValidatorsChangedEpoch !== null) {
        storedValidatorsRoot = await redis.getValidatorsRoot(latestValidatorsChangedEpoch);
      }
      await sleep(1000);
    }

    if (validatorsRoot !== storedValidatorsRoot) {
      console.log(`Validators roots for epoch ${epoch} differ: expected "${validatorsRoot}", got "${storedValidatorsRoot}"`);
      // reschedule tasks for epoch
      const changedValidators = await getValidatorsDiff(redis, beaconState, BigInt(epoch));
      await scheduler.saveValidatorsInBatches(changedValidators, BigInt(epoch));
      await redis.setValidatorsLength(BigInt(epoch), beaconState.validators.length);
      await redis.set(`${CONSTANTS.validatorsRootKey}:${epoch}`, validatorsRoot);
    }

    await redis.set(CONSTANTS.lastFinalizedEpochLookupKey, epoch.toString());
  } catch (error) {
    console.error(error);
  }
}

async function getFirstNonMissingSlotInEpoch(api: BeaconApi, epoch: number): Promise<number> {
  for (let relativeSlot = 0; relativeSlot < 31; ++relativeSlot) {
    const slot = epoch * 32 + relativeSlot;
    try {
      const status = await api.pingEndpoint(`/eth/v1/beacon/blocks/${slot}/root`);
      if (status === 200) {
        return slot;
      }
    } catch (error) {
      console.error(error);
    }
  }
  throw new Error("Did not find non-empty slot in epoch");
}

