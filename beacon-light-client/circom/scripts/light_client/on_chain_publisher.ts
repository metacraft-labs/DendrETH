import { ethers } from 'ethers';
import redisClient from './client';
import { postUpdateOnChain, PROOFS_CHANEL } from './relayer-helper';
import * as config from './config.json';
import { readFileSync } from 'fs';

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
    drainUpdatesInRedis();
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

    const keys = await redisClient.keys(`proof:${lastSlotOnChain}:*`);

    if (keys.length == 0) {
      return;
    }

    const proofResult = JSON.parse((await redisClient.get(keys[0]))!);

    try {
      await postUpdateOnChain(proofResult, lightClientContract);
      // Slow down broadcasting
      await new Promise(r => setTimeout(r, 2000));
    } catch (error) {
      console.log('ERROR occured in publishing transaction');
      console.log(error);
      console.log('will retry');
      await new Promise(r => setTimeout(r, 10000));
    }
  }
}


drainUpdatesInRedis();
