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
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: 'http://testing.mainnet.beacon-api.nimbus.team',
      description: 'Sets a custom beacon node url',
    })
    .option('sync-epoch', {
      alias: 'sync-epoch',
      describe: 'The sync epoch',
      type: 'number',
      default: undefined,
      description: 'Starts syncing from this epoch',
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

  const work_queue = new WorkQueue(
    new KeyPrefix(`${validator_commitment_constants.validatorProofsQueue}`),
  );

  const beaconApi = new BeaconApi([options['beacon-node']]);

  let headEpoch = BigInt(await beaconApi.getHeadSlot()) / 32n;
  let currentEpoch = options['sync-epoch'] !== undefined
    ? BigInt(options['sync-epoch']) : headEpoch;

  // handle zeros validators
  if (await redis.isZeroValidatorEmpty()) {
    console.log('Adding tasks about zeros');
    await redis.saveValidators([
      {
        index: Number(validator_commitment_constants.validatorRegistryLimit),
        data: {
          pubkey: ''.padEnd(96, '0'),
          withdrawalCredentials: ''.padEnd(64, '0'),
          effectiveBalance: ''.padEnd(64, '0'),
          slashed: ''.padEnd(64, '0'),
          activationEligibilityEpoch: ''.padEnd(64, '0'),
          activationEpoch: ''.padEnd(64, '0'),
          exitEpoch: ''.padEnd(64, '0'),
          withdrawableEpoch: ''.padEnd(64, '0'),
        },
      },
    ],
      currentEpoch,
    );

    await scheduleValidatorProof(BigInt(validator_commitment_constants.validatorRegistryLimit), currentEpoch);

    for (let level = 39n; level >= 0n; level--) {
      scheduleProveZeroForLevel(level);
      console.log('Added zeros tasks');
    }
  }

  console.log('Loading validators');
  let prevValidators = await redis.getValidatorsBatched(ssz);
  console.log('Loaded all batches');

  console.log(`syncing... ${currentEpoch}`);
  await updateValidators(currentEpoch);
  await syncEpoch();

  const es = await beaconApi.subscribeForEvents(['head']);
  es.on('head', async function (event) {
    headEpoch = BigInt(JSON.parse(event.data).slot) / 32n;

    await syncEpoch();
  })

  async function syncEpoch() {
    while (currentEpoch < headEpoch) {
      currentEpoch++;
      console.log(`syncing... ${currentEpoch === headEpoch ? currentEpoch : `${currentEpoch}/${headEpoch}`}`);
      await updateValidators(currentEpoch);
    }
  }

  async function updateValidators(epoch: bigint) {
    const { beaconState } = await beaconApi.getBeaconState(Number(epoch * 32n));
    const validators = beaconState.validators.slice(0, TAKE);

    const changedValidators = validators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(prevValidators));

    await saveValidatorsInBatches(epoch, changedValidators);

    console.log('#changedValidators', changedValidators.length);

    prevValidators = validators
  }

  async function saveValidatorsInBatches(epoch: bigint, validators: IndexedValidator[], batchSize = 200) {
    for (const batch of splitIntoBatches(validators, batchSize)) {
      await redis.saveValidators(
        batch.map((validator: IndexedValidator) => ({
          index: validator.index,
          data: convertValidatorToProof(validator.validator),
        })),
        epoch
      );
      await Promise.all(batch.map((validator) => scheduleValidatorProof(BigInt(validator.index), epoch)));
    }

    await updateBranches(epoch, validators);
  }

  async function scheduleValidatorProof(validatorIndex: bigint, epoch: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    dataView.setBigUint64(9, epoch, false);
    work_queue.addItem(db, new Item(buffer));

    // Don't create an epoch lookup for the zero validator proof
    if (validatorIndex !== BigInt(validator_commitment_constants.validatorRegistryLimit)) {
      await redis.addToEpochLookup(`${validator_commitment_constants.validatorProofKey}:${gindexFromValidatorIndex(validatorIndex)}`, epoch);
    }
  }

  async function scheduleUpdateProofNodeTask(gindex: bigint, epoch: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    await redis.addToEpochLookup(`${validator_commitment_constants.validatorProofKey}:${gindex}`, epoch);

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, epoch, false);
    work_queue.addItem(db, new Item(buffer));
  }

  function gindexFromValidatorIndex(index: bigint) {
    return (2n ** 40n) - 1n + index;
  }

  function getParent(gindex: bigint) {
    return (gindex - 1n) / 2n;
  }

  async function updateBranches(epoch: bigint, validators: IndexedValidator[]) {
    const changedValidatorGindices = validators.map(validator => gindexFromValidatorIndex(BigInt(validator.index)));

    let nodesNeedingUpdate = new Set(changedValidatorGindices.map(getParent));

    while (nodesNeedingUpdate.size !== 0) {
      const newNodesNeedingUpdate = new Set<bigint>();

      for (const gindex of nodesNeedingUpdate) {
        if (gindex !== 0n) {
          newNodesNeedingUpdate.add(getParent(gindex));
        }

        await redis.saveValidatorProof(gindex, epoch);
        await scheduleUpdateProofNodeTask(gindex, epoch);
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

  function convertValidatorToProof(validator: Validator) {
    return {
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
    };
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
