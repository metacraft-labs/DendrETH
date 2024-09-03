import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import CONSTANTS from '../../../kv_db_constants.json';
import { lightClean } from '../../light_cleaner_common';
import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import makeRedis from '../../utils/redis';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .withLightCleanOpts()
    .build();

  const redis: Redis = makeRedis(options);

  while (true) {
    console.log('Performing light clean');

    for (let depth = 0; depth <= 40; ++depth) {
      const prefix = new KeyPrefix(
        `${CONSTANTS.validatorProofsQueue}:${depth}`,
      );
      const queue = new WorkQueue(prefix);
      try {
        await lightClean.call(queue, redis, prefix);
      } catch {}
    }

    console.log(`Waiting ${options['clean-duration'] / 1000} seconds`);
    await sleep(options['clean-duration']);
  }
})();
