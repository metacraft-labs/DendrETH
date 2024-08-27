import { QueueEvents } from 'bullmq';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
} from '@/constants/constants';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

const config = {
  REDIS_HOST: process.env.REDIS_HOST || 'localhost',
  REDIS_PORT: Number(process.env.REDIS_PORT) || 6379,
};

checkConfig(config);

const proofGeneratorEvents = new QueueEvents(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.REDIS_HOST,
    port: config.REDIS_PORT,
  },
});

proofGeneratorEvents.on('failed', error => {
  logger.error('Proofing generation failed');

  logger.error(error);
});

const getUpdateEvents = new QueueEvents(UPDATE_POLING_QUEUE, {
  connection: {
    host: config.REDIS_HOST,
    port: config.REDIS_PORT,
  },
});

getUpdateEvents.on('failed', error => {
  logger.error('Error fetching update');

  logger.error(error);
});
