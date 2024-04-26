import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { lightClean } from '../light_cleaner_common';
import { CommandLineOptionsBuilder } from '../cmdline';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .withLightCleanOpts()
    .withProtocolOpts()
    .build();

  const redis = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );
  const queues: any[] = [];

  let protocol = options['protocol'];

  for (let i = 0; i < 39; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${protocol}:${validator_commitment_constants.balanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }

  while (true) {
    console.log('Performing light clean');

    for (let i = 0; i < queues.length; i++) {
      const prefix = new KeyPrefix(
        `${protocol}:${validator_commitment_constants.balanceVerificationQueue}:${i}`,
      );
      await lightClean.call(queues[i], redis, prefix);
    }

    console.log(`Waiting ${options['clean-duration'] / 1000} seconds`);
    await sleep(options['clean-duration']);
  }
})();
