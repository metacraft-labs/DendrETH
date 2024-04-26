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
    .build();

  const prefix = new KeyPrefix(
    `${validator_commitment_constants.validatorProofsQueue}`,
  );
  const validatorProofs = new WorkQueue(prefix);

  const redis = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  while (true) {
    console.log('Performing light clean');
    await lightClean.call(validatorProofs, redis, prefix);
    console.log(`Waiting ${options['clean-duration'] / 1000} seconds`);
    await sleep(options['clean-duration']);
  }
})();
