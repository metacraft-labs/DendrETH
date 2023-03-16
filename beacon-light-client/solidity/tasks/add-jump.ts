import { task } from 'hardhat/config';
import { Redis } from '../../../relay/implementations/redis';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Queue } from 'bullmq';
import { GetUpdate } from '../../../relay/types/types';
import { UPDATE_POLING_QUEUE } from '../../../relay/constants/constants';
import crypto from 'crypto';

task('add-jump', 'Add jump task')
  .addParam('startslot', 'the initial slot', undefined, undefined, false)
  .addParam('endslot', 'Slots to jump', undefined, undefined, false)
  .setAction(async args => {
    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    const lastDownloadedUpdateKey = crypto.randomUUID();

    await redis.set(lastDownloadedUpdateKey, args.startslot);

    const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
      connection: {
        host: config.REDIS_HOST!,
        port: Number(config.REDIS_PORT),
      },
    });

    await updateQueue.add(
      'downloadUpdate',
      {
        lastDownloadedUpdateKey: lastDownloadedUpdateKey,
        slotsJump: args.endslot - args.startslot,
      },
      {
        attempts: 10,
        backoff: {
          type: 'fixed',
          delay: 15000,
        },
      },
    );
  });
