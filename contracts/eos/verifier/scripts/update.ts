import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { getRootDir } from '../../../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

async function update(updateFile: string) {
  const rootDir = await getRootDir();
  const contractDir = rootDir + `/contracts/cosmos/verifier`;

  const pathToVerifyUtils =
    rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;

  const parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
  const parseUpdateDataCommand = `${parseDataTool} updateDataEOS \
      --proofPathEOS=${pathToVerifyUtils}proof_${updateFile} \
      --updatePathEOS=${pathToVerifyUtils}update_${updateFile}`;
  console.info(`Parsing data for update: \n ➤ ${parseUpdateDataCommand}`);
  const updateDataExec = exec(parseUpdateDataCommand);
  const updateData = (await updateDataExec).stdout.replace(/\s/g, '');

  const updateCommand =
    'cleos push action dendreth update ' + updateData + ' -p dendreth@active';
  const queryCommand = 'cleos get table dendreth dendreth verifierdata';
  const updateCommandExec = exec(updateCommand);
  const queryCommandExec = exec(queryCommand);
}

update('5200056_5200088.json');
