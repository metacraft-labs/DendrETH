import { Redis as RedisLocal } from '../../../relay/implementations/redis';

import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';

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

  const validatorKeys = await redis.getAllKeys(`${validator_commitment_constants.validatorKey}:*:${validator_commitment_constants.epochLookupKey}`);
  const validatorProofKeys = await redis.getAllKeys(`${validator_commitment_constants.validatorProofKey}:*:${validator_commitment_constants.epochLookupKey}`);
  const keys = [...validatorKeys, ...validatorProofKeys].map((key) => key.substring(0, key.lastIndexOf(':')));

  const deleted = await Promise.all(keys.map(async (key) => redis.pruneOldEpochs(key, options['oldest-epoch'])));
  const deletedCount = deleted.reduce((sum, value) => sum + value);
  console.log(`Deleted ${deletedCount} database entries`);
  await redis.disconnect();
})();
