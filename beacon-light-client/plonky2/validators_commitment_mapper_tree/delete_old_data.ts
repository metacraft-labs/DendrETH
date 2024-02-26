import { Redis as RedisLocal } from '../../../relay/implementations/redis';

import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';
import { createProofStorage } from '../proof_storage/proof_storage';

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
    .option('oldest-epoch', {
      alias: 'oldest-epoch',
      describe: 'The oldest epoch for which we keep data',
      type: 'number',
      demandOption: true,
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);
  const proofStorage = createProofStorage(options)

  let validatorKeys = await redis.getAllKeys(`${validator_commitment_constants.validatorKey}:*:${validator_commitment_constants.epochLookupKey}`);
  let validatorProofKeys = await redis.getAllKeys(`${validator_commitment_constants.validatorProofKey}:*:${validator_commitment_constants.epochLookupKey}`);

  validatorKeys = validatorKeys.map((key) => key.substring(0, key.lastIndexOf(':')));
  validatorProofKeys = validatorProofKeys.map((key) => key.substring(0, key.lastIndexOf(':')));

  // delete proofs
  const proofKeys = validatorProofKeys.map(key => validator_commitment_constants.validatorProofStorage + key.slice(key.indexOf(':')))
  await Promise.all(proofKeys.map(async (proofKey, index) => {
    const outdatedEpochs = await redis.collectOutdatedEpochs(validatorProofKeys[index], options['oldest-epoch'])
    const keysToDelete = outdatedEpochs.map(epoch => `${proofKey}:${epoch}`);
    return Promise.all(keysToDelete.map(key => proofStorage.delProof(key)));
  }));

  // delete redis data
  const redisKeys = [...validatorKeys, ...validatorProofKeys];
  const deleted = await Promise.all(redisKeys.map(async (key) => {
    return redis.pruneOldEpochs(key, options['oldest-epoch']);
  }));

  const deletedCount = deleted.reduce((sum, value) => sum + value);
  console.log(`Deleted ${deletedCount} database entries`);

  await proofStorage.quit();
  await redis.quit();
})();
