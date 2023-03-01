import { Queue, QueueEvents } from 'bullmq';
import {
  PROOF_GENERATOR_QUEUE,
  PUBLISH_ONCHAIN_QUEUE,
  UPDATE_POLING_QUEUE,
} from './relayer-helper';
import * as config from './config.json';

const updateQueue = new Queue<void>(UPDATE_POLING_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

updateQueue.add('downloadUpdate', undefined, {
  repeat: { every: config.updatePolingTime, immediately: true },
});

const proofGeneratorEvents = new QueueEvents(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

proofGeneratorEvents.on('failed', error => {
  console.error('Proofing generation failed');

  console.log(error);
});

const getUpdateEvents = new QueueEvents(UPDATE_POLING_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

getUpdateEvents.on('failed', error => {
  console.error('Error fetching update');

  console.log(error);
});

const publishOnChainEvents = new QueueEvents(PUBLISH_ONCHAIN_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

publishOnChainEvents.on('failed', error => {
  console.log('error while publishing on chain');

  console.log(error);
});
