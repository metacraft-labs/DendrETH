import {
  sleep,
  splitIntoBatches,
} from '../../../libs/typescript/ts-utils/common-utils';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Validator } from '../../../relay/types/types';
import { hexToBits } from '../../../libs/typescript/ts-utils/hex-utils';
import Redis from 'ioredis';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import { BeaconApi } from '../../../relay/implementations/beacon-api';

import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';

let TAKE: number | undefined;
let MOCK: boolean;

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
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: 'http://unstable.mainnet.beacon-api.nimbus.team',
      description: 'Sets a custom beacon node url',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: undefined,
      description: 'Sets the number of validators to take',
    })
    .option('mock', {
      alias: 'mock',
      describe: 'Runs the tool without doing actual calculations',
      type: 'boolean',
      default: false,
      description: 'Runs the tool without doing actual calculations.',
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];
  MOCK = options['mock'];

  const work_queue = new WorkQueue(
    new KeyPrefix(`${validator_commitment_constants.validatorProofsQueue}`),
  );

  const beaconApi = new BeaconApi([options['beacon-node']]);

  // handle zeros validators
  if (await redis.isZeroValidatorEmpty()) {
    console.log('Adding tasks about zeros');
    await redis.saveValidators([
      {
        index: Number(validator_commitment_constants.validatorRegistryLimit),
        validator: JSON.stringify({
          pubkey: Array(384).fill(0),
          withdrawalCredentials: Array(256).fill(0),
          effectiveBalance: Array(256).fill(0),
          slashed: Array(256).fill(0),
          activationEligibilityEpoch: Array(256).fill(0),
          activationEpoch: Array(256).fill(0),
          exitEpoch: Array(256).fill(0),
          withdrawableEpoch: Array(256).fill(0),
        }),
      },
    ]);

    const buffer = new ArrayBuffer(8);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(
      0,
      BigInt(validator_commitment_constants.validatorRegistryLimit),
      false,
    );

    await work_queue.addItem(db, new Item(buffer));

    for (let i = 0; i < 40; i++) {
      const buffer = new ArrayBuffer(24);
      const dataView = new DataView(buffer);

      dataView.setBigUint64(0, BigInt(i), false);
      dataView.setBigUint64(
        8,
        BigInt(validator_commitment_constants.validatorRegistryLimit),
        false,
      );
      dataView.setBigUint64(
        16,
        BigInt(validator_commitment_constants.validatorRegistryLimit),
        false,
      );

      await work_queue.addItem(db, new Item(buffer));

      console.log('Added zeros tasks');
    }
  }

  console.log('Loading validators');

  let prevValidators = await redis.getValidatorsBatched(ssz);

  console.log('Loaded all batches');

  while (true) {
    const timeBefore = Date.now();

    const validators = MOCK
      ? Array(TAKE).fill({
          pubkey: [
            147, 58, 217, 73, 27, 98, 5, 157, 208, 101, 181, 96, 210, 86, 216,
            149, 122, 140, 64, 44, 198, 232, 216, 238, 114, 144, 174, 17, 232,
            247, 50, 146, 103, 168, 129, 28, 57, 117, 41, 218, 197, 42, 225, 52,
            43, 165, 140, 149,
          ],
          withdrawalCredentials: [
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 54, 155, 180, 158, 250, 81,
            0, 253, 59, 134, 169, 248, 40, 197, 93, 160, 77, 45, 80,
          ],
          effectiveBalance: 32000000000,
          slashed: false,
          activationEligibilityEpoch: 0,
          activationEpoch: 0,
          exitEpoch: Infinity,
          withdrawableEpoch: Infinity,
        })
      : (await beaconApi.getValidators()).slice(0, TAKE);

    if (prevValidators.length === 0) {
      console.log('prev validators are empty. Saving to redis');

      const before = Date.now();

      await saveValidatorsInBatches(
        validators.map((validator, index) => ({
          index,
          validator,
        })),
      );

      const after = Date.now();

      console.log('Saved validators to redis');
      console.log('Time taken', after - before, 'ms');

      prevValidators = validators;

      if (!MOCK) {
        await sleep(384000);
      }
      continue;
    }

    const changedValidators = validators
      .map((validator, index) => ({ validator, index }))
      .filter(() => hasValidatorChanged(prevValidators));

    await saveValidatorsInBatches(changedValidators);

    console.log('#changedValidators', changedValidators.length);

    prevValidators = validators;

    const timeAfter = Date.now();

    // wait for the next epoch
    if (timeAfter - timeBefore < 384000) {
      await sleep(384000 - (timeBefore - timeAfter));
    }
  }

  async function saveValidatorsInBatches(
    validators: { index: number; validator: Validator }[],
    batchSize = 200,
  ) {
    const validatorBatches = splitIntoBatches(validators, batchSize);

    // Save each batch
    for (let i = 0; i < validatorBatches.length; i++) {
      await redis.saveValidators(
        validatorBatches[i].map(vi => ({
          index: vi.index,
          validator: convertValidatorToProof(vi.validator),
        })),
      );

      for (const vi of validatorBatches[i]) {
        const buffer = new ArrayBuffer(8);
        const dataView = new DataView(buffer);
        dataView.setBigUint64(0, BigInt(vi.index), false);
        await work_queue.addItem(db, new Item(buffer));
      }

      if (i % 25 == 0) {
        console.log('Saved 25 batches and added first level of proofs');
      }
    }

    if (validators.length > 0) {
      await addInnerLevelProofs(validators);
    }
  }

  async function addInnerLevelProofs(
    validators: { index: number; validator: Validator }[],
  ) {
    for (let j = 0n; j < 40n; j++) {
      console.log('Added inner level of proofs', j);

      let prev_index = 2199023255552n;
      for (let i = 0; i < validators.length; i++) {
        let validator_index = BigInt(validators[i].index);

        if (validator_index / 2n ** (j + 1n) == prev_index / 2n ** (j + 1n)) {
          continue;
        }

        const { first, second } = calculateIndexes(validator_index, j);

        const buffer = new ArrayBuffer(24);
        const dataView = new DataView(buffer);

        dataView.setBigUint64(0, BigInt(j), false);
        dataView.setBigUint64(8, first, false);
        dataView.setBigUint64(16, second, false);

        await redis.saveValidatorProof(j + 1n, first);

        await work_queue.addItem(db, new Item(buffer));

        prev_index = first;
      }
    }
  }

  function calculateIndexes(validator_index: bigint, depth: bigint) {
    let first: bigint, second: bigint;

    if (MOCK) {
      return { first: BigInt(0), second: BigInt(0) };
    }

    if (validator_index % 2n == 0n) {
      first = validator_index;
      second = validator_index + 1n;
    } else {
      first = validator_index - 1n;
      second = validator_index;
    }

    for (let k = 1n; k <= depth; k++) {
      if (first % 2n ** (k + 1n) == 0n) {
        second = first + 2n ** k;
      } else {
        second = first;
        first = first - 2n ** k;
      }
    }
    return { first, second };
  }

  function convertValidatorToProof(validator: Validator): string {
    return JSON.stringify({
      pubkey: hexToBits(bytesToHex(validator.pubkey), 381),
      withdrawalCredentials: hexToBits(
        bytesToHex(validator.withdrawalCredentials),
      ),
      effectiveBalance: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.effectiveBalance.hashTreeRoot(
            validator.effectiveBalance,
          ),
        ),
      ),
      slashed: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.slashed.hashTreeRoot(validator.slashed),
        ),
      ),
      activationEligibilityEpoch: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.activationEligibilityEpoch.hashTreeRoot(
            validator.activationEligibilityEpoch,
          ),
        ),
      ),
      activationEpoch: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.activationEpoch.hashTreeRoot(
            validator.activationEpoch,
          ),
        ),
      ),
      exitEpoch: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.exitEpoch.hashTreeRoot(
            validator.exitEpoch,
          ),
        ),
      ),
      withdrawableEpoch: hexToBits(
        bytesToHex(
          ssz.phase0.Validator.fields.withdrawableEpoch.hashTreeRoot(
            validator.withdrawableEpoch,
          ),
        ),
      ),
    });
  }

  function hasValidatorChanged(prevValidators) {
    return ({ validator, index }) =>
      prevValidators[index] === undefined ||
      validator.pubkey.some(
        (byte, i) => byte !== prevValidators[index].pubkey[i],
      ) ||
      validator.withdrawalCredentials.some(
        (byte, i) => byte !== prevValidators[index].withdrawalCredentials[i],
      ) ||
      validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
      validator.slashed !== prevValidators[index].slashed ||
      validator.activationEligibilityEpoch !==
        prevValidators[index].activationEligibilityEpoch ||
      validator.activationEpoch !== prevValidators[index].activationEpoch ||
      validator.exitEpoch !== prevValidators[index].exitEpoch ||
      validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch;
  }
})();
