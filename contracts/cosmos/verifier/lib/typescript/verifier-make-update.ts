import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { calculateFee, GasPrice } from '@cosmjs/stargate';
import { CosmosClientWithWallet } from '../../../../../libs/typescript/cosmos-utils/cosmos-utils';
import { getRootDir } from '../../../../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

export async function updateVerifierContract(
  network: string,
  cosmos: CosmosClientWithWallet,
  contractAddress: string,
  updateFile: string,
) {
  const rootDir = await getRootDir();
  const contractDir = rootDir + `/contracts/cosmos/verifier`;

  const pathToVerifyUtils =
    rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;

  const parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
  const parseUpdateDataCommand = `${parseDataTool} updateData \
      --proofPath=${pathToVerifyUtils}proof_${updateFile} \
      --updatePath=${pathToVerifyUtils}update_${updateFile}`;

  console.info(`Parsing data for update: \n ➤ ${parseUpdateDataCommand}`);
  const updateDataExec = exec(parseUpdateDataCommand);
  const updateData = (await updateDataExec).stdout.replace(/\s/g, '');

  let updateFee;

  switch (network) {
    case 'cudos': {
      console.info('Updating on Cudos Testnet');
      updateFee = 'auto';
      break;
    }
    case 'malaga': {
      console.info('Updating on Malaga Testnet');
      updateFee = 'auto';
      break;
    }
    case 'wasm': {
      console.info('Updating on local Testnet');
      const gasPrice = GasPrice.fromString('0.0000025ustake');
      updateFee = calculateFee(2_000_000, gasPrice);
      break;
    }
    default: {
      console.log('Incorrect network parameter');
    }
  }

  const client = cosmos.client;
  const DendrETHWalletInfo = cosmos.walletInfo;
  // Execute update on the contract with the contract specific message
  const result = await client.execute(
    DendrETHWalletInfo.address,
    contractAddress,
    JSON.parse(updateData),
    updateFee,
    'Updating the Verifier',
  );
  console.info(`\nUpdate result: `);
  console.log(result);
  return result;
}
