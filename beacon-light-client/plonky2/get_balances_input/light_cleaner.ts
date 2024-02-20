import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { getOptions, lightClean } from '../light_cleaner_common';

(async () => {
  const options = getOptions().argv;

  const redis = new Redis(
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
      const prefix = new KeyPrefix(
        `${validator_commitment_constants.balanceVerificationQueue}:${i}`,
      );
      await lightClean.call(queues[i], redis, prefix);
    }

    console.log(`Waiting ${options['clean-duration'] / 1000} seconds`);
    await sleep(options['clean-duration']);
  }
})();
