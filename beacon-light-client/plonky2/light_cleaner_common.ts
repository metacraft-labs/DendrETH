import Redis from 'ioredis';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

export async function lightClean(this: any, db: Redis, prefix: any) {
  const processingKey = prefix.of(':processing');
  const mainQueueKey = prefix.of(':queue');
  const cleaningKey = prefix.of(':cleaning');

  const processing: Array<string> = await db.lrange(processingKey, 0, -1);
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
      await db.rpush(mainQueueKey, itemId);
    }
    await db.lrem(cleaningKey, 0, itemId);
  }
}

export function getOptions() {
  return yargs(hideBin(process.argv))
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
    })
    .option('clean-duration', {
      alias: 'clean-duration',
      describe: 'The time between each clean in ms',
      type: 'number',
      default: 5000,
    });
}
