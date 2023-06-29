import { QueueEvents } from 'bullmq';
import { checkConfig } from '../libs/typescript/ts-utils/common-utils';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
} from './constants/constants';

const config = {
  REDIS_HOST: process.env.REDIS_HOST,
  REDIS_PORT: Number(process.env.REDIS_PORT),
};

checkConfig(config);

const proofGeneratorEvents = new QueueEvents(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.REDIS_HOST,
    port: config.REDIS_PORT,
  },
});

proofGeneratorEvents.on('failed', error => {
  console.error('Proofing generation failed');

  console.log(error);
});

const getUpdateEvents = new QueueEvents(UPDATE_POLING_QUEUE, {
  connection: {
    host: config.REDIS_HOST,
    port: config.REDIS_PORT,
  },
});

getUpdateEvents.on('failed', error => {
  console.error('Error fetching update');

  console.log(error);
});
