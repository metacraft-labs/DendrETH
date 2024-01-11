import {
  sleep,
  splitIntoBatches,
} from '../../../libs/typescript/ts-utils/common-utils';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Validator, IndexedValidator } from '../../../relay/types/types';
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

enum TaskTag {
  UPDATE_VALIDATOR_LEAF = 0,
  UPDATE_PROOF_NODE_TASK = 1,
  PROVE_ZERO_FOR_LEVEL = 2,
  UPDATE_VALIDATOR_PROOF = 3,
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
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: 'http://testing.mainnet.beacon-api.nimbus.team',
      description: 'Sets a custom beacon node url',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: undefined,
      description: 'Sets the number of validators to take',
    }).argv;

  /*
{
  const beaconApi = new BeaconApi([options['beacon-node']]);
  const { beaconState } = await beaconApi.getBeaconState(8172466);
  const validatorsRoot = ssz.capella.BeaconState.fields.validators.hashTreeRoot(beaconState.validators);
  const validatorsRootHex = bytesToHex(validatorsRoot);
  console.log(validatorsRootHex);
}
*/

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];

  const work_queue = new WorkQueue(
    new KeyPrefix(`${validator_commitment_constants.validatorProofsQueue}`),
  );

  const beaconApi = new BeaconApi([options['beacon-node']]);

  let epoch = 189000n;

  // handle zeros validators
  if (await redis.isZeroValidatorEmpty()) {
    console.log('Adding tasks about zeros');
    await redis.saveValidators([
      {
        index: Number(validator_commitment_constants.validatorRegistryLimit),
        validatorJSON: JSON.stringify({
          pubkey: ''.padEnd(96, '0'),
          withdrawalCredentials: ''.padEnd(64, '0'),
          effectiveBalance: ''.padEnd(64, '0'),
          slashed: ''.padEnd(64, '0'),
          activationEligibilityEpoch: ''.padEnd(64, '0'),
          activationEpoch: ''.padEnd(64, '0'),
          exitEpoch: ''.padEnd(64, '0'),
          withdrawableEpoch: ''.padEnd(64, '0'),
        }),
      },
    ]);

    scheduleValidatorProof(epoch, BigInt(validator_commitment_constants.validatorRegistryLimit));

    for (let level = 39n; level >= 0n; level--) {
      scheduleProveZeroForLevel(level);
      console.log('Added zeros tasks');
    }

  }

  console.log('Loading validators');

  let prevValidators = await redis.getValidatorsBatched(ssz);

  console.log('Loaded all batches');

  while (true) {
    const timeBefore = Date.now();

    const validators = await beaconApi.getValidators(8166208, TAKE);

    if (prevValidators.length === 0) {
      console.log('prev validators are empty. Saving to redis');

      const before = Date.now();

      await saveValidatorsInBatches(
        epoch,
        validators.map((validator, index) => ({
          index,
          validator,
        })),
      );

      const after = Date.now();

      console.log('Saved validators to redis');
      console.log('Time taken', after - before, 'ms');

      prevValidators = validators;

      await sleep(384000);
      continue;
    }

    const changedValidators = validators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(prevValidators));

    await saveValidatorsInBatches(epoch, changedValidators);

    console.log('#changedValidators', changedValidators.length);

    prevValidators = validators;

    const timeAfter = Date.now();

    // wait for the next epoch
    if (timeAfter - timeBefore < 384000) {
      await sleep(384000 - (timeBefore - timeAfter));
    }
  }

  async function saveValidatorsInBatches(epoch: bigint, validators: IndexedValidator[], batchSize = 200) {
    const validatorBatches = splitIntoBatches(validators, batchSize);

    // Save each batch
    validatorBatches.forEach(async (batch: IndexedValidator[], iteration: number) => {
      await redis.saveValidators(
        batch.map((validator: IndexedValidator) => ({
          index: validator.index,
          validatorJSON: convertValidatorToProof(validator.validator),
        })),
      );

      batch.forEach((validator: IndexedValidator) => scheduleValidatorProof(epoch, BigInt(validator.index)));

      if (iteration % 25 === 0) {
        console.log('Saved 25 batches and added first level of proofs');
      }

      await addInnerLevelProofs(epoch, validators);
    });
  }

  function scheduleValidatorProof(epoch: bigint, validatorIndex: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
    dataView.setBigUint64(1, epoch, false);
    dataView.setBigUint64(9, validatorIndex, false);
    work_queue.addItem(db, new Item(buffer));
  }

  function scheduleUpdateProofNodeTask(epoch: bigint, gindex: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE_TASK);
    dataView.setBigUint64(1, epoch, false);
    dataView.setBigUint64(9, gindex, false);

    work_queue.addItem(db, new Item(buffer));
  }

  function gindexFromValidatorIndex(index: bigint) {
    return (2n ** 40n) - 1n + index;
  }

  function getNthParent(gindex: bigint, n: bigint) {
    return (gindex - (2n ** n) + 1n) / (2n ** n);
  }

  async function addInnerLevelProofs(epoch: bigint, validators: IndexedValidator[]) {
    const changedValidatorGindices = validators.map(validator => gindexFromValidatorIndex(BigInt(validator.index)));

    let nodesNeedingUpdate = new Set<bigint>();
    changedValidatorGindices.forEach(gindex => {
      nodesNeedingUpdate.add(getNthParent(gindex, 1n));
    });

    while (nodesNeedingUpdate.size !== 0) {
      const newNodesNeedingUpdate = new Set<bigint>();

      for (const gindex of nodesNeedingUpdate) {
        if (gindex !== 0n) {
          newNodesNeedingUpdate.add(getNthParent(gindex, 1n));
        }

        await redis.saveValidatorProof(gindex, epoch);
        scheduleUpdateProofNodeTask(epoch, gindex);
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

  function convertValidatorToProof(validator: Validator): string {
    return JSON.stringify({
      pubkey: bytesToHex(validator.pubkey),
      withdrawalCredentials: bytesToHex(validator.withdrawalCredentials),
      effectiveBalance: bytesToHex(
        ssz.phase0.Validator.fields.effectiveBalance.hashTreeRoot(
          validator.effectiveBalance,
        ),
      ),
      slashed: bytesToHex(
        ssz.phase0.Validator.fields.slashed.hashTreeRoot(validator.slashed),
      ),
      activationEligibilityEpoch: bytesToHex(
        ssz.phase0.Validator.fields.activationEligibilityEpoch.hashTreeRoot(
          validator.activationEligibilityEpoch,
        ),
      ),
      activationEpoch: bytesToHex(
        ssz.phase0.Validator.fields.activationEpoch.hashTreeRoot(
          validator.activationEpoch,
        ),
      ),
      exitEpoch: bytesToHex(
        ssz.phase0.Validator.fields.exitEpoch.hashTreeRoot(validator.exitEpoch),
      ),
      withdrawableEpoch: bytesToHex(
        ssz.phase0.Validator.fields.withdrawableEpoch.hashTreeRoot(
          validator.withdrawableEpoch,
        ),
      ),
    });
  }

  function hasValidatorChanged(prevValidators: Validator[]) {
    return ({ validator, index }: IndexedValidator) =>
      prevValidators[index] === undefined
      || validator.pubkey.some((byte, i) => byte !== prevValidators[index].pubkey[i])
      || validator.withdrawalCredentials.some((byte, i) => byte !== prevValidators[index].withdrawalCredentials[i])
      || validator.effectiveBalance !== prevValidators[index].effectiveBalance
      || validator.slashed !== prevValidators[index].slashed
      || validator.activationEligibilityEpoch !== prevValidators[index].activationEligibilityEpoch
      || validator.activationEpoch !== prevValidators[index].activationEpoch
      || validator.exitEpoch !== prevValidators[index].exitEpoch
      || validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch;
  }
})();
