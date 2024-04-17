import Redis from 'ioredis';

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
