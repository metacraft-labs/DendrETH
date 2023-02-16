import { Queue, QueueEvents } from 'bullmq';
import { readFile, writeFile } from 'fs/promises';
import {
  PROOF_GENERATOR_QUEUE,
  State,
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
  repeat: { every: config.updatePolingTime },
});

const proofGeneratorEvents = new QueueEvents(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

proofGeneratorEvents.on('completed', async (job, data) => {
  // here will publish proof on chain
  while (true) {
    const state: State = JSON.parse(await readFile('state.json', 'utf-8'));
    let reutrnValue = job.returnvalue as any;
    if (reutrnValue.prevUpdateSlot === state.lastUpdateOnChain) {
      state.lastUpdateOnChain = reutrnValue.updateSlot;
      await writeFile('state.json', JSON.stringify(state));
      console.log('WORK DONE');
      return;
    } else {
      // WAIT UNTIL IT IS TIME FOR YOU POLL EVERY 5 seconds
      await new Promise(r => setTimeout(r, 5000));
    }
  }
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
