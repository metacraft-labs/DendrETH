import { Queue, QueueEvents } from 'bullmq';
import {
  PROOF_GENERATOR_QUEUE,
  UPDATE_POLING_QUEUE,
  getProofKey,
  GetUpdate,
  postUpdateOnChain,
  PROOFS_CHANEL,
} from './relayer-helper';
import * as config from './config.json';
import { ethers } from 'ethers';
import { readFileSync } from 'fs';
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
    `downloadUpdate:${config.lightClientAddress}`,
    {
      lastDownloadedUpdateKey: `lastDownloadedUpdateKey:${config.lightClientAddress}`,
      slotsJump: config.slotsJump,
    },
    {
      repeat: { every: config.slotsJump * 12000, immediately: true },
    },
  );

  drainUpdatesInRedis();
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

const provider = new ethers.providers.JsonRpcProvider(
  config.etherJsonRpcProvider,
);

const wallet = new ethers.Wallet(config.privateKey, provider);

const light_client_abi = JSON.parse(
  readFileSync('./light_client.abi.json', 'utf-8'),
);

const lightClientContract = new ethers.Contract(
  config.lightClientAddress,
  light_client_abi,
  wallet,
);

const subscriber = redisClient.duplicate();

subscriber.connect().then(() => {
  subscriber.subscribe(PROOFS_CHANEL, async () => {
    try {
      drainUpdatesInRedis();
    } catch (error) {
      console.log('ERROR occured in publishing transaction');
      console.log(error);
    }
  });
});

async function drainUpdatesInRedis() {
  while (true) {
    const header_root_on_chain =
      await lightClientContract.optimistic_header_root();

    console.log(header_root_on_chain);

    const onChainHeaderResult = await (
      await fetch(
        `http://${config.beaconRestApiHost}:${config.beaconRestApiPort}/eth/v1/beacon/headers/${header_root_on_chain}`,
      )
    ).json();

    const lastSlotOnChain = Number(
      onChainHeaderResult.data.header.message.slot,
    );

    const proofResult = JSON.parse(
      (await redisClient.get(
        getProofKey(lastSlotOnChain, lastSlotOnChain + config.slotsJump),
      ))!,
    );

    if (proofResult === null) {
      return;
    }

    await postUpdateOnChain(proofResult, lightClientContract);
  }
}
