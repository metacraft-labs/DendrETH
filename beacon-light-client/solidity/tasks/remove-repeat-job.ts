import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Queue } from 'bullmq';
import { GetUpdate } from '@dendreth/relay/types/types';
import { UPDATE_POLING_QUEUE } from '@dendreth/relay/constants/constants';
import { task } from 'hardhat/config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

task('remove-repeat-job', 'Run update recuring task')
  .addParam('jobKey', 'The job key')
  .setAction(async args => {
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

    logger.info(await updateQueue.getRepeatableJobs());

    await updateQueue.removeRepeatableByKey(args.jobKey);
  });
