import Redis from 'ioredis';
import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';

(async () => {
  const db = new Redis('redis://127.0.0.1:6379');
  const queue = new WorkQueue(
    new KeyPrefix(
      `${validator_commitment_constants.balanceVerificationQueue}:${0}`,
    ),
  );

  const result = await queue.leaseExists(db, '0');

  console.log(result);
})();
