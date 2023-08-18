import { getFilesInDir } from '../../../../libs/typescript/ts-utils/data';
import { getProof } from './get_light_client_recursive_input';
import * as path from 'path';
import { readFileSync, writeFileSync } from 'fs';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as vkey from './converted-vkey.json';
import { getGenericLogger } from '../../../../libs/typescript/ts-utils/logger';

const logger = getGenericLogger();
const promiseExec = promisify(exec);
const network = 'mainnet';

const proofsDir = path.join(
  __dirname,
  `../../../../vendor/eth2-light-client-updates/${network}/recursive-proofs`,
);

const UPDATES = getFilesInDir(
  path.join(
    __dirname,
    '../../../../',
    'vendor',
    'eth2-light-client-updates',
    'mainnet',
    'updates',
  ),
);
let prevUpdate = UPDATES[0];
let period = 291;

(async () => {
  for (let update of UPDATES.slice(1)) {
    logger.info('Proof convertion...');
    await promiseExec(
      `python ${path.join(__dirname, 'proof_converter.py')} ${proofsDir}/proof${
        period - 1
      }.json ${proofsDir}/public${period - 1}.json`,
    );
    logger.info('Input generation...');
    const proof = JSON.parse(readFileSync(`proof.json`).toString());

    writeFileSync(
      path.join(__dirname, 'input.json'),
      JSON.stringify(
        await getProof(
          vkey,
          proof,
          [
            '5966029082507805980254291345114545245067072315222408966008558171151621124246',
            '4',
          ],
          JSON.parse(prevUpdate.toString()),
          JSON.parse(update as unknown as string),
        ),
      ),
    );

    logger.info('Witness generation...');
    logger.info(
      await promiseExec(
        `${path.join(
          __dirname,
          '../../build/light_client_recursive/light_client_recursive_cpp/light_client_recursive',
        )} ${path.join(__dirname, 'input.json')} witness.wtns`,
      ),
    );

    logger.info('Proof generation...');
    logger.info(
      await promiseExec(
        `${path.join(
          __dirname,
          '../../../../vendor/rapidsnark/build/prover',
        )} ${path.join(
          __dirname,
          '../../build/light_client_recursive/light_client_recursive_0.zkey',
        )} witness.wtns ${proofsDir}/proof${period}.json ${proofsDir}/public${period}.json`,
      ),
    );

    period++;
    await promiseExec(
      `rm ${path.join(__dirname, 'input.json')} witness.wtns proof.json`,
    );
    prevUpdate = update;
  }
})();
