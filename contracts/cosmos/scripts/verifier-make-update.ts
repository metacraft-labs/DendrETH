import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';

const gasPrice = GasPrice.fromString('0.0000025acudos');

const exec = promisify(exec_);

// ender addr and update

var DendrETHWalletInfo = {
  mnemonic: '',
  address: '',
};
//TODO: Use Dimo's configuration check func
const _cudosContractAddress = String(process.env['CUDOS_CONTRACT_ADDRESS']);
const _cudosMnemonic = String(process.env['CUDOS_MNEMONIC']);
const _cudosPublicAddress = String(process.env['CUDOS_PUBLIC_KEY']);
let client: SigningCosmWasmClient;
var rpcEndpoint = 'http://localhost:26657';

let updateNum = '5200120_5200152.json';
var wallet;
async function Update() {
  const readline = require('readline').createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  let answer1 = await new Promise(resolve => {
    readline.question('Enter network(cosmosTestnet): ', resolve);
  });
  const network = process.argv[2];

  switch (network) {
    case 'cosmosTestnet': {
      console.info('Uploading to Cosmos Testnet');
      (DendrETHWalletInfo.mnemonic = _cudosMnemonic),
        (rpcEndpoint = 'https://explorer.public-testnet.fl.cudos.org:36657/');

      let answer2 = await new Promise(resolve => {
        readline.question(
          'Do we use cudos13k5ktkd6lzvegzwrx8nxmxu5u4pqj7z8tzfszm as address for cosmosTestnet? ',
          resolve,
        );
      });
      if (!answer2) {
        let answer2: string = await new Promise(resolve => {
          readline.question('Enter address: ', resolve);
        });
        DendrETHWalletInfo.address = answer2;
      } else {
        DendrETHWalletInfo.address = _cudosPublicAddress;
      }

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
          gasPrice: GasPrice.fromString('10000000000000acudos'),
          // gasPrice: GasPrice.fromString('10000000000000000000acudos'),
        },
      );

      break;
    }
    default: {
      DendrETHWalletInfo.mnemonic =
        'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone';
      const getFredAddressCommand = `wasmd keys show fred -a --keyring-backend test \
      --keyring-dir $HOME/.wasmd_keys`;
      const getAddress = exec(getFredAddressCommand);
      const fredAddress = (await getAddress).stdout;
      wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        DendrETHWalletInfo.mnemonic,
        {
          prefix: 'wasm',
        },
      );
      DendrETHWalletInfo.address = fredAddress.trimEnd();
      client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        wallet,
      );
    }
  }

  const rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
    /\s/g,
    '',
  );

  const contractDirVerifier = rootDir + `/contracts/cosmos/verifier`;
  const parseDataTool = `${contractDirVerifier}/nimcache/verifier_parse_data`;
  const pathToVerifyUtils =
    rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
  const parseUpdateDataCommand = `${parseDataTool} updateData \
      --proofPath=${pathToVerifyUtils}proof_${updateNum} --updatePath=${pathToVerifyUtils}update_${updateNum}`;
  console.info(`Parsing data for update 1: \n ➤ ${parseUpdateDataCommand}`);
  const updateDataExec = exec(parseUpdateDataCommand);
  const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
  console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);

  // Execute update on the contract with the contract specific message
  const executeFee = calculateFee(2_000_000, gasPrice);
  const result = await client.execute(
    DendrETHWalletInfo.address,
    _cudosContractAddress,
    JSON.parse(updateData),
    'auto',
    'Updating the Verifier',
  );
  console.log(result);
}

Update();
