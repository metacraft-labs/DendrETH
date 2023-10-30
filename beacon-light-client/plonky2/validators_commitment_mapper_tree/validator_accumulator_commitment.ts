// Should accept initial validators as inputs from a file maybe
// Should be able somehow to get info about new validators from the smart contract maybe
// Should be able to return merkle proofs and so on

import {
  sleep,
  splitIntoBatches,
} from '../../../libs/typescript/ts-utils/common-utils';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Validator } from '../../../relay/types/types';
import Redis from 'ioredis';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import { BeaconApi } from '../../../relay/implementations/beacon-api';

import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';
import { readFileSync } from 'fs';

let TAKE: number | undefined;

enum TaskTag {
  UPDATE_PROOF_NODE = 0,
  PROVE_ZERO_FOR_LEVEL = 1,
  UPDATE_VALIDATOR_PROOF = 2,
}

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .option('redis-host ', {
      alias: 'redis-host',
      describe: 'The Redis host',
      type: 'string',
      default: '127.0.0.1',
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: 6379,
      description: 'Sets a custom redis connection',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: undefined,
      description: 'Sets the number of validators to take',
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];

  console.log("WTF");

  const work_queue = new WorkQueue(
    new KeyPrefix(
      `${validator_commitment_constants.validatorAccumulatorProofQueue}`,
    ),
  );

  // handle zeros validators
  if (await redis.isZeroValidatorEmpty()) {
    console.log('Adding tasks about zeros');
    await redis.saveAccumulatorValidators([
      {
        index: Number(validator_commitment_constants.validatorRegistryLimit),
        data: {
          validator_pubkey: ''.padEnd(96, '0'),
          eth1_deposit_index: 0,
        },
      },
    ]);

    await scheduleValidatorProof(
      BigInt(validator_commitment_constants.validatorRegistryLimit),
    );

    for (let level = 39n; level >= 0n; level--) {
      scheduleProveZeroForLevel(level);
      console.log('Added zeros tasks');
    }
  }

  // load from file
  let validators: [
    { validator_pubkey: string; validator_eth1_deposit_index: number },
  ] = JSON.parse(readFileSync('./validators.json', 'utf8'));

  saveValidatorsInBatches(validators);

  async function saveValidatorsInBatches(
    validators: [
      { validator_pubkey: string; validator_eth1_deposit_index: number },
    ],
    batchSize = 200,
  ) {
    await Promise.all(
      splitIntoBatches(validators, batchSize).map(async batch => {
        await redis.saveAccumulatorValidators(
          batch.map((validator, index) => ({
            index: index,
            data: validator,
          })),
        );

        await Promise.all(
          batch.map((_, index) =>
            scheduleValidatorProof(BigInt(index)),
          ),
        );
      }),
    );

    await updateBranches(validators);
  }

  async function scheduleValidatorProof(validatorIndex: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    work_queue.addItem(db, new Item(buffer));
  }

  async function scheduleUpdateProofNodeTask(gindex: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE);
    dataView.setBigUint64(1, gindex, false);
    work_queue.addItem(db, new Item(buffer));
  }

  function gindexFromValidatorIndex(index: bigint) {
    return 2n ** 40n - 1n + index;
  }

  function getParent(gindex: bigint) {
    return (gindex - 1n) / 2n;
  }

  async function updateBranches(validators: [{ validator_pubkey: string; validator_eth1_deposit_index: number }]) {
    const changedValidatorGindices = validators.map((_, index) =>
      gindexFromValidatorIndex(BigInt(index)),
    );

    let nodesNeedingUpdate = new Set(changedValidatorGindices.map(getParent));

    while (nodesNeedingUpdate.size !== 0) {
      const newNodesNeedingUpdate = new Set<bigint>();

      for (const gindex of nodesNeedingUpdate) {
        if (gindex !== 0n) {
          newNodesNeedingUpdate.add(getParent(gindex));
        }

        await redis.saveValidatorAccumulatorProof(gindex);
        await scheduleUpdateProofNodeTask(gindex);
      }

      nodesNeedingUpdate = newNodesNeedingUpdate;
    }
  }

  function scheduleProveZeroForLevel(level: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.PROVE_ZERO_FOR_LEVEL);
    dataView.setBigUint64(1, level, false);

    work_queue.addItem(db, new Item(buffer));
  }
})();
