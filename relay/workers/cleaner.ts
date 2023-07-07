import { Queue } from 'bullmq';
import {
  checkConfig,
  sleep,
} from '../../libs/typescript/ts-utils/common-utils';
import { GetUpdate } from '../types/types';
import { UPDATE_POLING_QUEUE } from '../constants/constants';

(async () => {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(config);

  const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
    connection: {
      host: config.REDIS_HOST!,
      port: Number(config.REDIS_PORT),
    },
  });

  while (true) {
    console.log('cleaner running');
    const waitingJobs = await updateQueue.getWaiting();

    const hashSet = new Set<string>();

    for (const job of waitingJobs) {
      if (hashSet.has(JSON.stringify([job.data.from, job.data.to]))) {
        console.log('job removed');
        await job.remove().catch(e => console.log(e));
      } else {
        hashSet.add(JSON.stringify([job.data.from, job.data.to]));
      }
    }

    await sleep(12000);
  }
})();
