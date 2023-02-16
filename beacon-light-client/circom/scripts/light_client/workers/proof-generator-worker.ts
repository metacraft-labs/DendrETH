import { Worker } from 'bullmq';
import { exec as _exec } from 'child_process';
import { rm, writeFile } from 'fs/promises';
import path from 'path';
import { promisify } from 'util';
import {
  ProofInputType,
  RELAYER_WITNESSES_FOLDER,
  PROOF_GENERATOR_QUEUE,
  RELAYER_INPUTS_FOLDER,
} from '../relayer-helper';
import * as config from '../config.json';

const exec = promisify(_exec);

new Worker<ProofInputType>(
  PROOF_GENERATOR_QUEUE,
  async job => {
    await writeFile(
      path.join(
        __dirname,
        '..',
        RELAYER_INPUTS_FOLDER,
        `input_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
      ),
      JSON.stringify(job.data.proofInput),
    );

    await exec(
      `${config.witnessGeneratorPath} ${path.join(
        __dirname,
        '..',
        RELAYER_INPUTS_FOLDER,
        `input_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
      )} ${path.join(
        __dirname,
        '..',
        RELAYER_WITNESSES_FOLDER,
        `witness_${job.data.prevUpdateSlot}_${job.data.updateSlot}.wtns`,
      )}`,
    );

    // await exec(
    //   `${RAPIDSNARK_PROVER_PATH} ${ZKEY_FILE_PATH} ${path.join(
    //     RELAYER_WITNESSES_FOLDER,
    //     `witness_${job.data.prevUpdateSlot}_${job.data.updateSlot}.wtns`,
    //   )} ${path.join(
    //     RELAYER_PROOFS_FOLDER,
    //     `proof_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
    //   )} ${path.join(
    //     RELAYER_PROOFS_FOLDER,
    //     `public_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
    //   )}`,
    // );

    // remove witness as it is huge unneeded file
    await rm(
      path.join(
        __dirname,
        '..',
        RELAYER_WITNESSES_FOLDER,
        `witness_${job.data.prevUpdateSlot}_${job.data.updateSlot}.wtns`,
      ),
    );

    return {
      prevUpdateSlot: job.data.prevUpdateSlot,
      updateSlot: job.data.updateSlot,
    };
  },
  {
    connection: {
      host: config.redisHost,
      port: config.redisPort,
    },
    concurrency: 4,
  },
);
