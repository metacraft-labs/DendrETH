import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { setUpCosmosTestnet } from '../../../tests/cosmosLightClient/helpers/testnet-setup';

const exec = promisify(exec_);

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

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
  var instantiateFee;

  const network = process.argv[2];
  console.log('ARGS: ', process.argv);

  switch (network) {
    case 'cudos': {
      console.info('Uploading to Cudos Testnet');
      DendrETHWalletInfo = {
        mnemonic: String(process.env['KUDOS_MNEMONIC']),
        address: String(process.env['KUDOS_PUBLIC_KEY']),
      };
      rpcEndpoint = 'https://explorer.public-testnet.fl.cudos.org:36657/';
      gasPrice = GasPrice.fromString('0.0000025acudos');
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
        },
      );
      uploadFee = 'auto';
      instantiateFee = 'auto';
      break;
    }
    case 'local': {
      console.info('Uploading to Local Testnet');
      rpcEndpoint = 'http://localhost:26657';
      gasPrice = GasPrice.fromString('0.0000025ustake');
      let cosmos = await setUpCosmosTestnet(rootDir, rpcEndpoint, signal);
      await sleep(10000);

      client = cosmos.client;
      DendrETHWalletInfo = cosmos.DendrETHWalletInfo;
      uploadFee = calculateFee(1_500_000, gasPrice);
      instantiateFee = calculateFee(2_000_000, gasPrice);

      break;
    }
    default: {
      console.log('Incorrect network parameter');
    }
  }

  const wasm = fs.readFileSync(nimwasmPath);
  const uploadReceipt = await client.upload(
    DendrETHWalletInfo.address,
    wasm,
    uploadFee,
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

  const initDataExec = exec(parseInitDataCommand);
  const initData = (await initDataExec).stdout.replace(/\s/g, '');
  console.info(`Parsed instantiation data: \n  ╰─➤ ${initData}`);
  const instantiation = await client.instantiate(
    DendrETHWalletInfo.address,
    uploadReceipt.codeId,
    JSON.parse(initData),
    'My instance',
    instantiateFee,
    { memo: 'Create a Verifier in Cosmos instance.' },
  );
  var _contractAddress = instantiation.contractAddress;
  const queryResultAfterInitialization = await client.queryContractSmart(
    _contractAddress,
    {
      last_header_hash: {},
    },
  );

  const header = queryResultAfterInitialization.toString().replace(/\s/g, '');
  console.info(
    '\n Initiation of contract complete with init header hash: ',
    header,
    _contractAddress,
  );
}

UploadMain();
