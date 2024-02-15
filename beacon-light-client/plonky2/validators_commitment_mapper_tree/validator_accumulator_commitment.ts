import { splitIntoBatches } from '../../../libs/typescript/ts-utils/common-utils';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import {
  IndexedValidatorPubkeyDeposit,
  ValidatorPubkeyDeposit,
} from '../../../relay/types/types';
import Redis from 'ioredis';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');

import CONSTANTS from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';
import { readFileSync } from 'fs';
import chalk from 'chalk';
import { makeBranchIterator } from './utils';

let TAKE: number | undefined;

enum TaskTag {
  APPEND_VALIDATOR_ACCUMULATOR_PROOF = 0,
  UPDATE_PROOF_NODE = 1,
  PROVE_ZERO_FOR_DEPTH = 2,
}

(async () => {
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
    })
    .option('protocol', {
      alias: 'protocol',
      describe: 'The protocol',
      type: 'string',
      default: 'demo',
      description: 'Sets the protocol',
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];

  const queue = new WorkQueue(
    new KeyPrefix(`${CONSTANTS.validatorAccumulatorProofQueue}`),
  );

  if (await redis.isZeroValidatorAccumulatorEmpty()) {
    console.log(chalk.bold.blue('Adding zero tasks...'));
    await scheduleZeroTasks();
  }

  let validators = await redis.getValidatorsAccumulatorBatched(
    options['protocol'],
  );

  validators = await updateValidators(validators);

  console.log('Done');

  // TODO: LISTEN for ValidatorsAccumulator events

  async function saveValidatorsInBatches(
    validators: IndexedValidatorPubkeyDeposit[],
    batchSize = 200,
  ) {
    await Promise.all(
      splitIntoBatches(validators, batchSize).map(async batch => {
        await redis.saveAccumulatorValidators(batch, options['protocol']);

        await Promise.all(
          batch.map((_, index) => scheduleValidatorProof(BigInt(index))),
        );
      }),
    );

    let levelIterator = makeBranchIterator(
      validators.map((_, index) => BigInt(index)),
      32n,
    );

    let leafs = levelIterator.next().value!;

    await Promise.all(
      leafs.map(gindex =>
        redis.saveValidatorAccumulatorProof(gindex, options['protocol']),
      ),
    );

    for (const gindices of levelIterator) {
      await Promise.all(
        gindices.map(gindex =>
          redis.saveValidatorAccumulatorProof(gindex, options['protocol']),
        ),
      );
      await Promise.all(
        gindices.map(gindex => scheduleUpdateProofNodeTask(gindex)),
      );
    }
  }

  async function scheduleValidatorProof(validatorIndex: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.APPEND_VALIDATOR_ACCUMULATOR_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    queue.addItem(db, new Item(buffer));
  }

  async function scheduleUpdateProofNodeTask(gindex: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE);
    dataView.setBigUint64(1, gindex, false);
    queue.addItem(db, new Item(buffer));
  }

  async function updateValidators(oldValidators: ValidatorPubkeyDeposit[]) {
    // TODO: think of better way
    // load from file
    let validators: [
      { validator_pubkey: string; validator_eth1_deposit_index: number },
    ] = JSON.parse(readFileSync('./validators.json', 'utf8'));

    const changedValidators = validators
      .map((validator, index) => ({ validator, index }))
      .filter((_, index) => index >= oldValidators.length);

    await saveValidatorsInBatches(changedValidators);

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );

    return validators;
  }

  async function scheduleZeroTasks() {
    await redis.saveAccumulatorValidators(
      [
        {
          index: Number(CONSTANTS.validatorRegistryLimit),
          validator: {
            validator_pubkey: ''.padEnd(96, '0'),
            validator_eth1_deposit_index: 0,
          },
        },
      ],
      options['protocol'],
    );

    await scheduleValidatorProof(BigInt(CONSTANTS.validatorRegistryLimit));
    await redis.saveZeroValidatorAccumulatorProof(32n);

    for (let depth = 31n; depth >= 0n; depth--) {
      scheduleProveZeroForDepth(depth);
      await redis.saveZeroValidatorAccumulatorProof(depth);
    }
  }

  async function scheduleProveZeroForDepth(depth: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.PROVE_ZERO_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    queue.addItem(redis.client, new Item(buffer));
  }
})();
