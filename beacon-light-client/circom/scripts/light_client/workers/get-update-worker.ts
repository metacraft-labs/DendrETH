import { Worker, Queue } from 'bullmq';
import { exec as _exec } from 'child_process';
import { writeFile } from 'fs/promises';
import path from 'path';
import {
  ProofInputType,
  PROOF_GENERATOR_QUEUE,
  RELAYER_INPUTS_FOLDER,
  UPDATE_POLING_QUEUE,
  GetUpdate,
} from '../relayer-helper';
import * as config from '../config.json';
import { getInputFromTo } from '../get_light_client_input_from_to';
import redisClient from '../client';

const proofGenertorQueue = new Queue<ProofInputType>(PROOF_GENERATOR_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

new Worker<GetUpdate>(
  UPDATE_POLING_QUEUE,
  async job => {
    console.log('WTF');

    const currentHeadResult = await (
      await fetch(
        `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v1/beacon/headers/head`,
      )
    ).json();

    const currentHeadSlot = Number(currentHeadResult.data.header.message.slot);

    const lastDownloadedUpdate = Number.parseInt(
      (await redisClient.get(job.data.lastDownloadedUpdateKey))!,
    );

    if (currentHeadSlot <= lastDownloadedUpdate + job.data.slotsJump) {
      console.log('No new enought slot');
      return;
    }

    const result = await getInputFromTo(
      lastDownloadedUpdate,
      lastDownloadedUpdate + job.data.slotsJump,
      {
        beaconRestApiHost: config.beaconRestApiHost,
        beaconRestApiPort: config.beaconRestApiPort,
      },
    );

    await writeFile(
      path.join(
        __dirname,
        '..',
        RELAYER_INPUTS_FOLDER,
        `input_${result.prevUpdateSlot}_${result.updateSlot}.json`,
      ),
      JSON.stringify(result.proofInput),
    );

    await proofGenertorQueue.add('proofGenerate', result);

    await redisClient.set(job.data.lastDownloadedUpdateKey, result.updateSlot);
  },
  {
    connection: {
      host: config.redisHost,
      port: config.redisPort,
    },
  },
);
