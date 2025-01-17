import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import Redis from 'ioredis';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import CONSTANTS from '../../../kv_db_constants.json'; // PESHO
import { lightClean } from '../../../light_cleaner_common';

interface LightCleanParams {
  redis: Redis;
  protocol: string;
  cleanDuration: number;
  silent: boolean;
}

export async function lightCleanQueue(params: LightCleanParams) {
  const queues: any[] = [];

  for (let i = 0; i < 32; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${params.protocol}:${CONSTANTS.depositBalanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }

  while (true) {
    if (!params.silent) {
      console.log('Performing light clean');
    }

    for (let i = 0; i < queues.length; i++) {
      const prefix = new KeyPrefix(
        `${params.protocol}:${CONSTANTS.depositBalanceVerificationQueue}:${i}`,
      );
      await lightClean.call(queues[i], params.redis, prefix);
    }

    await sleep(params.cleanDuration);
  }
}
