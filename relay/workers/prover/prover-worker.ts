import { Worker } from 'bullmq';
import { exec as _exec } from 'child_process';
import { ProofInputType } from '../../types/types';
import genProof from './gen_proof';
import { Redis } from '../../implementations/redis';
import { Prover } from '../../implementations/prover';
import { PROOF_GENERATOR_QUEUE } from '../../constants/constants';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';

(async () => {
  const proverConfig = {
    WITNESS_GENERATOR_PATH: process.env.WITNESS_GENERATOR_PATH,
    RAPIDSNAKR_PROVER_PATH: process.env.RAPIDSNAKR_PROVER_PATH,
    ZKEY_FILE_PATH: process.env.ZKEY_FILE_PATH,
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(proverConfig);

  const redis = new Redis(proverConfig.REDIS_HOST!, proverConfig.REDIS_PORT);

  const prover = new Prover(
    proverConfig.WITNESS_GENERATOR_PATH!,
    proverConfig.RAPIDSNAKR_PROVER_PATH!,
    proverConfig.ZKEY_FILE_PATH!,
  );

  new Worker<ProofInputType>(
    PROOF_GENERATOR_QUEUE,
    async job => genProof(redis, prover, job.data),
    {
      connection: {
        host: proverConfig.REDIS_HOST,
        port: proverConfig.REDIS_PORT,
      },
      concurrency: 1,
    },
  );
})();
