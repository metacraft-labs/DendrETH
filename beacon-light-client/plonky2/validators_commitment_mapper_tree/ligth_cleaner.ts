const {
  KeyPrefix,
  WorkQueue,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

(async () => {
  const options = yargs(hideBin(process.argv))
    .usage('Usage: -redis-host <Redis host> -redis-port <Redis port>')
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
    }).argv;

  const first_level_proofs = new WorkQueue(
    new KeyPrefix(`${validator_commitment_constants.validatorProofsQueue}`),
  );

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  while (true) {
    console.log('Performing light clean');
    await first_level_proofs.lightClean(db);
    console.log('Waiting 5 seconds');
    await sleep(5000);
  }
})();
