import { Queue, QueueEvents } from 'bullmq';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
  GetUpdate,
} from './relayer-helper';
import * as config from './config.json';
import redisClient from './client';

const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

(async () => {
  const lastDownloadedUpdateKey = `lastDownloadedUpdateKey:${config.lightClientAddress}`;

  await redisClient.set(lastDownloadedUpdateKey, config.startingSlot);

  await updateQueue.add(
    'downloadUpdate',
    {
      lastDownloadedUpdateKey: `lastDownloadedUpdateKey:${config.lightClientAddress}`,
      slotsJump: config.slotsJump,
    },
    {
      repeat: { every: config.slotsJump * 12000, immediately: true },
    },
  );
})();

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
