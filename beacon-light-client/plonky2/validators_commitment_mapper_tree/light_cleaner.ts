const {
  KeyPrefix,
  WorkQueue,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import { sleep } from '../../../libs/typescript/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';

async function lightCleanMod(this: any, db: Redis, prefix: any) {
  const processingKey = prefix.of(":processing");
  const mainQueueKey = prefix.of(":queue");
  const cleaningKey = prefix.of(":cleaning");


  const processing: Array<string> = await db.lrange(
    processingKey,
    0,
    -1
  );
  for (const itemId of processing) {
    if (!(await this.leaseExists(db, itemId))) {
      await db.lpush(cleaningKey, itemId);
      const removed = await db.lrem(processingKey, 0, itemId);
      if (removed > 0) {
        await db.rpush(mainQueueKey, itemId);
      }
      await db.lrem(cleaningKey, 0, itemId);
    }
  }

  const forgot: Array<string> = await db.lrange(cleaningKey, 0, -1);
  for (const itemId of forgot) {
    const leaseExists: boolean = await this.leaseExists(db, itemId);
    if (
      !leaseExists &&
      (await db.lpos(mainQueueKey, itemId)) == null &&
      (await db.lpos(processingKey, itemId)) == null
    ) {
      /**
       * FIXME: this introduces a race
       * maybe not anymore
       * no, it still does, what if the job has been completed?
       */
      await db.rpush(mainQueueKey, itemId);
    }
    await db.lrem(cleaningKey, 0, itemId);
  }
}

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

  const prefix = new KeyPrefix(`${validator_commitment_constants.validatorProofsQueue}`);
  const first_level_proofs = new WorkQueue(prefix);

  const redis = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  while (true) {
    console.log('Performing light clean');
    await lightCleanMod.call(first_level_proofs, redis, prefix);
    console.log('Waiting 5 seconds');
    await sleep(5000);
  }
})();
