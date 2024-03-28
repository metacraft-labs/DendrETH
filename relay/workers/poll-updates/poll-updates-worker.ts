import { Worker, Queue } from 'bullmq';
import { exec as _exec } from 'child_process';
import { GetUpdate, ProofInputType } from '../../types/types';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
} from '../../constants/constants';
import doUpdate from './do_update';
import { getBeaconApi } from '../../implementations/beacon-api';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { initPrometheusSetup } from '@dendreth/utils/ts-utils/prometheus-utils';

const logger = getGenericLogger();
initPrometheusSetup();

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

  new Worker<GetUpdate>(
    UPDATE_POLING_QUEUE,
    async job => {
      await doUpdate(
        await getBeaconApi(job.data.networkConfig.BEACON_REST_API),
        proofGenertorQueue,
        job.data.from,
        job.data.to,
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
