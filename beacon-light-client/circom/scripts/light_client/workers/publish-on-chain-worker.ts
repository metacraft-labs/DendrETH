import { Worker } from 'bullmq';
import { ethers } from 'ethers';
import { readFileSync } from 'fs';
import { readFile, writeFile } from 'fs/promises';
import * as config from '../config.json';
import {
  ProofResultType,
  PUBLISH_ONCHAIN_QUEUE,
  State,
} from '../relayer-helper';
import { groth16 } from 'snarkjs';

const provider = new ethers.providers.JsonRpcProvider(
  config.etherJsonRpcProvider,
);

const wallet = new ethers.Wallet(config.privateKey, provider);

const light_client_abi = JSON.parse(
  readFileSync('../light_client.abi.json', 'utf-8'),
);

const lightClientContract = new ethers.Contract(
  config.lightClientAddress,
  light_client_abi,
  wallet,
);

new Worker<ProofResultType>(
  PUBLISH_ONCHAIN_QUEUE,
  async job => {
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

      if (lastSlotOnChain > job.data.prevUpdateSlot) {
        return;
      }

      if (lastSlotOnChain === job.data.prevUpdateSlot) {
        const calldata = await groth16.exportSolidityCallData(
          job.data.proof,
          job.data.proof.public,
        );

        const argv: string[] = calldata
          .replace(/["[\]\s]/g, '')
          .split(',')
          .map(x => BigInt(x).toString());

        const a = [argv[0], argv[1]];
        const b = [
          [argv[2], argv[3]],
          [argv[4], argv[5]],
        ];
        const c = [argv[6], argv[7]];

        const transaction = await lightClientContract.light_client_update({
          attested_header_root:
            '0x' +
            BigInt('0b' + job.data.proofInput.nextHeaderHash.join(''))
              .toString(16)
              .padStart(64, '0'),
          finalized_header_root:
            '0x' +
            BigInt('0b' + job.data.proofInput.finalizedHeaderRoot.join(''))
              .toString(16)
              .padStart(64, '0'),
          finalized_execution_state_root:
            '0x' +
            BigInt('0b' + job.data.proofInput.execution_state_root.join(''))
              .toString(16)
              .padStart(64, '0'),
          a: a,
          b: b,
          c: c,
        });

        console.log(transaction);

        await transaction.wait();

        const state: State = JSON.parse(
          await readFile('../state.json', 'utf-8'),
        );
        state.lastUpdateOnChain = job.data.updateSlot;
        await writeFile('../state.json', JSON.stringify(state));
        return;
      } else {
        // WAIT FOR THE UPDATE
        await new Promise(r => setTimeout(r, 15000));
      }
    }
  },
  {
    concurrency: 100,
    connection: {
      host: config.redisHost,
      port: config.redisPort,
    },
  },
);
