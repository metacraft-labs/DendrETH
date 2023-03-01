import { Worker, Queue } from 'bullmq';
import { exec as _exec } from 'child_process';
import { readFile, rm, writeFile } from 'fs/promises';
import path from 'path';
import { promisify } from 'util';
import {
  ProofInputType,
  RELAYER_WITNESSES_FOLDER,
  PROOF_GENERATOR_QUEUE,
  RELAYER_INPUTS_FOLDER,
  RELAYER_PROOFS_FOLDER,
  PUBLISH_ONCHAIN_QUEUE,
  ProofResultType,
} from '../relayer-helper';
import * as config from '../config.json';

const exec = promisify(_exec);

const proofPublishQueue = new Queue<ProofResultType>(PUBLISH_ONCHAIN_QUEUE, {
  connection: {
    host: config.redisHost,
    port: config.redisPort,
  },
});

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

    await exec(
      `${config.rapidSnarkProverPath} ${config.zkeyFilePath} ${path.join(
        __dirname,
        '..',
        RELAYER_WITNESSES_FOLDER,
        `witness_${job.data.prevUpdateSlot}_${job.data.updateSlot}.wtns`,
      )} ${path.join(
        __dirname,
        '..',
        RELAYER_PROOFS_FOLDER,
        `proof_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
      )} ${path.join(
        __dirname,
        '..',
        RELAYER_PROOFS_FOLDER,
        `public_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
      )}`,
    );

    // remove witness as it is huge unneeded file
    await rm(
      path.join(
        __dirname,
        '..',
        RELAYER_WITNESSES_FOLDER,
        `witness_${job.data.prevUpdateSlot}_${job.data.updateSlot}.wtns`,
      ),
    );

    const proof = JSON.parse(
      await readFile(
        path.join(
          __dirname,
          '..',
          RELAYER_PROOFS_FOLDER,
          `proof_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
        ),
        'utf-8',
      ),
    );

    const publicVars = JSON.parse(
      await readFile(
        path.join(
          __dirname,
          '..',
          RELAYER_PROOFS_FOLDER,
          `public_${job.data.prevUpdateSlot}_${job.data.updateSlot}.json`,
        ),
        'utf-8',
      ),
    );

    await proofPublishQueue.add('proofGenerate', {
      prevUpdateSlot: job.data.prevUpdateSlot,
      updateSlot: job.data.updateSlot,
      proofInput: job.data.proofInput,
      proof: {
        pi_a: proof.pi_a,
        pi_b: proof.pi_b,
        pi_c: proof.pi_c,
        public: publicVars,
      },
    });
  },
  {
    connection: {
      host: config.redisHost,
      port: config.redisPort,
    },
    concurrency: 1,
  },
);
