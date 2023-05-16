import * as fs from 'fs';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { UploadResult } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';

import {
  CosmosClientWithWallet as CosmosClientWithWallet,
  getCosmosContractArtifacts,
} from '../../../../../libs/typescript/cosmos-utils/cosmos-utils';

const exec = promisify(exec_);

let instantiateFee;

export async function uploadVerifierContract(
  network: string,
  cosmos: CosmosClientWithWallet,
) {
  const { wasmContractPath } = await getCosmosContractArtifacts('verifier');
  const contract = fs.readFileSync(wasmContractPath);
  const { client, walletInfo: DendrETHWalletInfo } = cosmos;

  let uploadFee;
  switch (network) {
    case 'cudos': {
      uploadFee = 'auto';
      instantiateFee = 'auto';
      console.info('Uploading to Cudos network');
      break;
    }
    case 'malaga': {
      uploadFee = 'auto';
      instantiateFee = 'auto';
      console.info('Uploading to Malaga network');
      break;
    }
    case 'wasm': {
      const gasPrice = GasPrice.fromString('0.0000025ustake');
      instantiateFee = calculateFee(2_000_000, gasPrice);
      uploadFee = calculateFee(1_500_000, gasPrice);
      console.info('Uploading to Local Testnet');
      break;
    }
    default: {
      console.error('Incorrect network parameter');
      break;
    }
  }
  const uploadReceipt = await client.upload(
    DendrETHWalletInfo.address,
    contract,
    uploadFee,
    'Upload `Verifier` contract',
  );
  return uploadReceipt;
}

export async function instantiateVerifierContract(
  uploadReceipt: UploadResult,
  initHeaderRoot: string,
  domain: string,
  cosmos: CosmosClientWithWallet,
) {
  const { rootDir, contractDir } = await getCosmosContractArtifacts('verifier');

  const pathToVerifyUtils =
    rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates-94/`;
  const pathToKey = pathToVerifyUtils + `vk.json`;

  const parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
  const parseInitDataCommand = `${parseDataTool} initData \
  --initHeaderRoot=${initHeaderRoot} \
  --domain=${domain} \
  --verificationKeyPath=${pathToKey}`;

  console.info(
    `Parsing data for instantiation. \n  ╰─➤ ${parseInitDataCommand}`,
  );
  const initDataExec = exec(parseInitDataCommand);
  const initData = (await initDataExec).stdout.replace(/\s/g, '');
  console.info(`Parsed instantiation data: \n  ╰─➤ ${initData}`);

  const client = cosmos.client;
  const DendrETHWalletInfo = cosmos.walletInfo;

  console.info('Instantiating contract');
  const instantiation = await client.instantiate(
    DendrETHWalletInfo.address,
    uploadReceipt.codeId,
    JSON.parse(initData),
    'Instantiating the `Verifier`',
    instantiateFee,
    { memo: 'Create a `Verifier` instance.' },
  );
  let contractAddress = instantiation.contractAddress;
  const queryResultAfterInitialization = await client.queryContractSmart(
    contractAddress,
    {
      last_header_hash: {},
    },
  );

  const header = queryResultAfterInitialization.toString().replace(/\s/g, '');
  console.info(
    '\n Initiation of contract completed',
    { header },
    { contractAddress },
  );
  return instantiation;
}
