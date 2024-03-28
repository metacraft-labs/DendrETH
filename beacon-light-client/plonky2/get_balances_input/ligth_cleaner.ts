const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';

(async () => {
  const options = yargs
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

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );
  const queues: any[] = [];

  for (let i = 0; i < 39; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${validator_commitment_constants.balanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }
  while (true) {
    console.log('Performing light clean');

    for (let i = 0; i < 39; i++) {
      await queues[i].lightClean(db);
    }

    console.log('Waiting 5 seconds');
    await sleep(5000);
  }
})();
