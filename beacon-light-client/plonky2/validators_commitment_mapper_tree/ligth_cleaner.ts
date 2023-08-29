const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import { sleep } from '../../../libs/typescript/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';

(async () => {
  const first_level_proofs = new WorkQueue(
    new KeyPrefix(
      `${validator_commitment_constants.validatorProofsQueue}`,
    ),
  );
  const db = new Redis('redis://127.0.0.1:6379');

  while (true) {
    console.log('Performing light clean');
    await first_level_proofs.lightClean(db);
    console.log('Waiting 5 seconds');
    await sleep(5000);
  }
})();
