import { Worker, Queue } from 'bullmq';
import { exec as _exec } from 'child_process';
import { GetUpdate, ProofInputType } from '../../types/types';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
} from '../../constants/constants';
import doUpdate from './do_update';
import { Redis } from '../../implementations/redis';
import { BeaconApi } from '../../implementations/beacon-api';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';

(async () => {
  const updatePollingConfig = {
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(updatePollingConfig);

  const proofGenertorQueue = new Queue<ProofInputType>(PROOF_GENERATOR_QUEUE, {
    connection: {
      host: updatePollingConfig.REDIS_HOST,
      port: updatePollingConfig.REDIS_PORT,
    },
  });

  const redis = new Redis(
    updatePollingConfig.REDIS_HOST!,
    updatePollingConfig.REDIS_PORT,
  );

  const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
    connection: {
      host: updatePollingConfig.REDIS_HOST!,
      port: updatePollingConfig.REDIS_PORT,
    },
  });

  const activeJobs = await updateQueue.getActive();

  new Worker<GetUpdate>(
    UPDATE_POLING_QUEUE,
    async job => {
      for (let activeJob of activeJobs) {
        if (
          activeJob.data.beaconRestApi == job.data.beaconRestApi &&
          activeJob.data.slotsJump == job.data.slotsJump
        ) {
          return;
        }
      }

      doUpdate(
        redis,
        new BeaconApi([job.data.beaconRestApi]),
        proofGenertorQueue,
        job.data.lastDownloadedUpdateKey,
        job.data.slotsJump,
        job.data.networkConfig,
      );
    },
    {
      connection: {
        host: updatePollingConfig.REDIS_HOST,
        port: updatePollingConfig.REDIS_PORT,
      },
      concurrency: 1,
    },
  );
})();
