import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Queue } from 'bullmq';
import { GetUpdate } from '../../../relay/types/types';
import { UPDATE_POLING_QUEUE } from '../../../relay/constants/constants';
import { task } from 'hardhat/config';

task('remove-repeat-job', 'Run update recuring task')
  .addParam('jobkey', 'The job key')
  .setAction(async args => {
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

    console.log(await updateQueue.getRepeatableJobs());

    await updateQueue.removeRepeatableByKey(args.jobkey);
  });
