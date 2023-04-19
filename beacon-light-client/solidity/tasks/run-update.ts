import { task } from 'hardhat/config';
import { Redis } from '../../../relay/implementations/redis';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Queue } from 'bullmq';
import { GetUpdate } from '../../../relay/types/types';
import {
  Config,
  UPDATE_POLING_QUEUE,
} from '../../../relay/constants/constants';
import * as networkConfig from '../../../relay/constants/network_config.json';

task('run-update', 'Run update recuring task')
  .addParam(
    'lightclient',
    'The address of the BeaconLightClient contract',
    undefined,
    undefined,
    true,
  )
  .addParam('initialslot', 'the initial slot', undefined, undefined, false)
  .addParam('slotsjump', 'Slots to jump', undefined, undefined, false)
  .addParam('follownetwork')
  .setAction(async args => {
    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    if (!networkConfig[args.follownetwork]) {
      console.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = networkConfig[args.follownetwork] as Config;

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    const lastDownloadedUpdateKey = !args.lightclient
      ? `lastDownloadedUpdateKey:${currentConfig.BEACON_REST_API}`
      : `lastDownloadedUpdateKey:${currentConfig.BEACON_REST_API}:${args.lightclient}`;

    const downloadUpdate = !args.lightclient
      ? `downloadUpdate${currentConfig.BEACON_REST_API}`
      : `downloadUpdate${currentConfig.BEACON_REST_API}${args.lightclient}`;

    await redis.set(lastDownloadedUpdateKey, args.initialslot);

    const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
      connection: {
        host: config.REDIS_HOST!,
        port: Number(config.REDIS_PORT),
      },
    });

    await updateQueue.add(
      downloadUpdate,
      {
        lastDownloadedUpdateKey: lastDownloadedUpdateKey,
        beaconRestApi: currentConfig.BEACON_REST_API,
        slotsJump: Number(args.slotsjump),
        networkConfig: currentConfig,
      },
      {
        attempts: 10,
        backoff: {
          type: 'fixed',
          delay: 15000,
        },
        repeat: { every: args.slotsjump * 12000, immediately: true },
      },
    );
  });
