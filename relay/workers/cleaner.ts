import { Queue } from 'bullmq';
import { checkConfig, sleep } from '@dendreth/utils/ts-utils/common-utils';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { GetUpdate } from '@/types/types';
import { UPDATE_POLING_QUEUE } from '@/constants/constants';

const logger = getGenericLogger();

(async () => {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST || 'localhost',
    REDIS_PORT: Number(process.env.REDIS_PORT) || 6379,
  };

  checkConfig(config);

  const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
    connection: {
      host: config.REDIS_HOST!,
      port: Number(config.REDIS_PORT),
    },
  });

  while (true) {
    logger.info('cleaner running');
    const waitingJobs = await updateQueue.getWaiting();

    const hashSet = new Set<string>();

    for (const job of waitingJobs) {
      if (hashSet.has(JSON.stringify([job.data.from, job.data.to]))) {
        logger.info('job removed');
        await job.remove().catch(e => logger.info(e));
      } else {
        hashSet.add(JSON.stringify([job.data.from, job.data.to]));
      }
    }

    await sleep(12000);
  }
})();
