import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
// import * as readline from 'readline';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { setUpCosmosTestnet } from '../../../tests/cosmosLightClient/helpers/testnet-setup';
import { resolve } from 'node:path';
import env from 'hardhat';
// import { network } from 'hardhat';
const exec = promisify(exec_);
var gasPrice;
const controller = new AbortController();
const { signal } = controller;
let DendrETHWalletInfo = {
  mnemonic:
    'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone',
  address: '',
};
let client: SigningCosmWasmClient;
var nimwasmPath =
  '/home/mcl-kristin/code/mklabs/DendrETH/contracts/cosmos/verifier/artifacts/verifier.wasm';
async function UploadMain() {
  const rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
    /\s/g,
    '',
  );
  var rpcEndpoint;
  var wallet;
  var uploadFee;
  const readline = require('readline').createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  let answer = await new Promise(resolve => {
    readline.question('Enter network(cosmosTestnet): ', resolve);
  });
  const network = answer;

  switch (network) {
    case 'cosmosTestnet': {
      console.info('Uploading to Cosmos Testnet');
      DendrETHWalletInfo = {
        mnemonic: String(process.env['KUDOS_MNEMONIC']),
        address: '',
      };
      rpcEndpoint = 'https://explorer.public-testnet.fl.cudos.org:36657/';
      gasPrice = GasPrice.fromString('0.0000025acudos');
      DendrETHWalletInfo.address = String(process.env['KUDOS_PUBLIC_KEY']);
      wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        DendrETHWalletInfo.mnemonic,
        {
          prefix: 'cudos',
        },
      );
      client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        wallet,
        {
          gasPrice: GasPrice.fromString('5000000000000acudos'),

          // gasPrice: GasPrice.fromString('7500000000000000000acudos'),
        },
      );
      // uploadFee;
      // 15066830000000000000000000acudos:
      // 150668300000000000000000acudos
      // 7533415000000000000acudos
      // 15066830000000acudos
      break;
    }
    default: {
      // console.info('Uploading to Local Testnet');
      // rpcEndpoint = 'http://localhost:26657';
      // gasPrice = GasPrice.fromString('0.0000025ustake');
      // let cosmos = await setUpCosmosTestnet(rootDir, rpcEndpoint, signal);
      // client = cosmos.client;
      // DendrETHWalletInfo = cosmos.DendrETHWalletInfo;
      // uploadFee = calculateFee(1_500_000, gasPrice);
    }
  }

  const wasm = fs.readFileSync(nimwasmPath);
  const uploadReceipt = await client.upload(
    DendrETHWalletInfo.address,
    wasm,
    'auto',
    'Upload Verifier in Cosmos contract',
  );
  var contractDirVerifier = rootDir + `/contracts/cosmos/verifier`;
  var parseDataTool = `${contractDirVerifier}/nimcache/verifier_parse_data`;
  var pathToVerifyUtils =
    rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
  var pathToKey = pathToVerifyUtils + `vkey.json`;
  var pathToFirstHeader = pathToVerifyUtils + `update_5200024_5200056.json`;
  const parseInitDataCommand = `${parseDataTool} initData \
  --initHeaderPath=${pathToFirstHeader} \
  --verificationKeyPath=${pathToKey}`;
  console.info(
    `Parsing data for instantiation. \n  ╰─➤ ${parseInitDataCommand}`,
  );
  const instantiateFee = calculateFee(2_000_000, gasPrice);

  const updateDataExec = exec(parseInitDataCommand);
  const initData = (await updateDataExec).stdout.replace(/\s/g, '');
  console.info(`Parsed instantiation data: \n  ╰─➤ ${initData}`);
  const instantiation = await client.instantiate(
    DendrETHWalletInfo.address,
    uploadReceipt.codeId,
    JSON.parse(initData),
    'My instance',
    'auto',
    { memo: 'Create a Verifier in Cosmos instance.' },
  );
  var _contractAddress = instantiation.contractAddress;
  const queryResultAfterInitialization = await client.queryContractSmart(
    _contractAddress,
    {
      header: {},
    },
  );

  const header = queryResultAfterInitialization.toString().replace(/\s/g, '');
  console.info(
    '\n Initiation of contract complete with init header hash: ',
    header,
    _contractAddress,
  );
}
//       7.1337520000000000000acudos
// 11300122.500000000000000000acudos
UploadMain();
// async function hello() {
//   const readline = require('readline').createInterface({
//     input: process.stdin,
//     output: process.stdout,
//   });

//   const answer = await new Promise(resolve => {
//     readline.question('What is your name? ', resolve);
//   });
//   console.log(answer);

//   console.log('THIS IS TEST');
// }

// hello();
