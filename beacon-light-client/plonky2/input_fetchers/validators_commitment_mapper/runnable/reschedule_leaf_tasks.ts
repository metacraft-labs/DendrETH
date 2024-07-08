import CONSTANTS from '../../../kv_db_constants.json';
import { Item, KeyPrefix, WorkQueue } from "@mevitae/redis-work-queue";
import { CommandLineOptionsBuilder } from "../../utils/cmdline";
import { Redis } from "@dendreth/relay/implementations/redis";
import { lightClean } from "../../light_cleaner_common";
import assert from "assert";
import { ChainableCommander } from "ioredis";
import { CommitmentMapperScheduler } from "../lib/scheduler";
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import fs from 'fs';

(async function() {
  const args = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .withBeaconNodeOpts()
    .option('clean', {
      type: 'boolean'
    })
    .option('push', {
      type: 'boolean'
    })
    .option('value', {
      type: 'string'
    })
    .option('lease-out', {
      type: 'boolean'
    })
    .build();

  const basePath = '../crates/circuit_executables/output';
  const nonVerifiableIndicesContent = fs.readFileSync(`${basePath}/non_verifiable_indices_0_1450497_9332704.txt`).toString();
  const missingIndicesContent = fs.readFileSync(`${basePath}/missing_indices_0_1450497_9332704.txt`).toString();

  const nonVerifiableIndices: number[] = JSON.parse(nonVerifiableIndicesContent);
  const mapping: { [key: number]: number[] } = {};

  for (const validatorIndex of nonVerifiableIndices) {
    const gindex = 2 ** 40 + validatorIndex;
    const parentGindex = Math.floor(gindex / 2);
    if (!mapping[parentGindex]) {
      mapping[parentGindex] = [];
    }
    mapping[parentGindex].push(validatorIndex);
  }

  // const buggedInnerLevelGIndices: number[] = [];
  // for (const key in mapping) {
  //   const children = mapping[key];
  //   if (children.length > 1) {
  //     buggedInnerLevelGIndices.push(Number(key));
  //   }
  // }

  // console.log(buggedInnerLevelGIndices);
  // console.log(buggedInnerLevelGIndices.length);

  const missingIndices: number[] = JSON.parse(missingIndicesContent);

  console.log(nonVerifiableIndices);
  // console.log(missingIndices);

  const combinedIndices = nonVerifiableIndices.concat(missingIndices);
  console.log(combinedIndices);
  console.log('len', combinedIndices.length);

  const index = combinedIndices.reverse().findIndex((idx) => idx == 1266346);
  console.log(index);

  return;

  const redis = new Redis(args['redis-host'], args['redis-port']);
  console.log('connected to redis');

  const prefix = new KeyPrefix('validator_proof_queue');
  const slot = 9332704n;

  const pipeline = redis.client.pipeline();

  for (const index of combinedIndices) {
    await scheduleValidatorProof(pipeline, prefix, BigInt(index), slot);
  }

  console.log('executing command');
  await pipeline.exec();

  await redis.quit();
})();

function pushItemInFront(pipeline: ChainableCommander, prefix: KeyPrefix, item: Item) {
  const itemDataKey = new KeyPrefix(prefix.of(':item:'));
  const mainQueueKey = prefix.of(':queue');

  pipeline.set(itemDataKey.of(item.id), item.data);
  pipeline.rpush(mainQueueKey, item.id);
}

async function scheduleValidatorProof(pipeline: ChainableCommander, prefix: KeyPrefix, validatorIndex: bigint, slot: bigint) {
  const buffer = new ArrayBuffer(17);
  const dataView = new DataView(buffer);
  dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
  dataView.setBigUint64(1, validatorIndex, false);
  dataView.setBigUint64(9, slot, false);

  const item = new Item(Buffer.from(buffer));
  pushItemInFront(pipeline, prefix, item);
}

async function getValidatorsNeedingHashing(redis: Redis) {
  const allKeys = await redis.client.keys('validator_proof:*');
  const keys = allKeys.filter((key) => !key.includes('slot_lookup'));
  const proofs = keys.filter((key) => {
    const parts = key.split(':');
    const gindex = Number(parts[1]);
    const slot = Number(parts[2]);
    return slot == 9332704 && gindex >= 2n ** 40n;
  });

  const jsons: any[] = [];
  for (const key of proofs) {
    const parts = key.split(':');
    const gindex = Number(parts[1]);
    const index = gindex - 2 ** 40;
    const json = await redis.get(key);
    jsons.push([index, json]);
  }
  const objs = jsons.map(([index, json]) => [index, JSON.parse(json!)]);
  objs.sort(([index1, _1], [index2, _2]) => index1 - index2);

  const uncalculatedLeaves = objs.filter(([_, leaf]) => leaf.needsChange);
  return uncalculatedLeaves.map((entry) => entry[0]);
}

enum TaskTag {
  UPDATE_PROOF_NODE = 0,
  PROVE_ZERO_FOR_DEPTH = 1,
  UPDATE_VALIDATOR_PROOF = 2,
  ZERO_OUT_VALIDATOR = 3,
}

function range(size: number, startAt: number = 0) {
  return [...Array(size).keys()].map(i => i + startAt);
}
