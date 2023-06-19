const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';

(async () => {
  const validator_registry_limit = 1099511627776n;

  const proofs_queue = new WorkQueue(new KeyPrefix('validator_proofs'));

  const db = new Redis('redis://localhost:6379');

  while ((await proofs_queue.queueLen(db)) > 0) {
    let item = await proofs_queue.lease(db, 30);

    console.log(item.data);

    await proofs_queue.complete(db, item);
  }

  console.log('done');

  const redis = new RedisLocal('localhost', 6379);

  await redis.saveValidators([
    {
      index: Number(validator_registry_limit),
      validator: JSON.stringify({
        pubkey: Array(384).fill(false),
        withdrawalCredentials: Array(256).fill(false),
        effectiveBalance: Array(256).fill(false),
        slashed: Array(256).fill(false),
        activationEligibilityEpoch: Array(256).fill(false),
        activationEpoch: Array(256).fill(false),
        exitEpoch: Array(256).fill(false),
        withdrawableEpoch: Array(256).fill(false),
      }),
    },
  ]);

  console.log('Saved validators to redis');

  // Add the first level proofs to the queue
  {
    const buffer = new ArrayBuffer(8);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, validator_registry_limit, false);

    await proofs_queue.addItem(db, new Item(buffer));

    console.log('Added first task to queue');
  }

  for (let i = 0; i < 41; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, BigInt(i), false);
    dataView.setBigUint64(8, validator_registry_limit, false);
    dataView.setBigUint64(16, validator_registry_limit, false);

    await proofs_queue.addItem(db, new Item(buffer));

    console.log('Added task to queue');
  }

  console.log('Done adding tasks to queue');
})();
