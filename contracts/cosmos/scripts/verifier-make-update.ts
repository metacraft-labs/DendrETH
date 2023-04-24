import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';

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

// Need to pass this by name or ...?
let updateNum = '5200024_5200056.json';
var wallet;
async function Update() {
  const network = process.argv[2];
  const contractAddress = process.argv[3];
  console.log('ARGS: ', process.argv);
  var updateFee;

  switch (network) {
    case 'cosmosTestnet': {
      console.info('Updating on Cudos Testnet');
      (DendrETHWalletInfo.mnemonic = _cudosMnemonic),
        (rpcEndpoint = 'https://explorer.public-testnet.fl.cudos.org:36657/');

      const cudosAddress = process.argv[4];
      DendrETHWalletInfo.address = cudosAddress;

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
        },
      );
      // const gasPrice = GasPrice.fromString('0.0000025acudos');
      updateFee = 'auto';
      break;
    }
    case 'local': {
      console.info('Updating on local Testnet');
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
      const gasPrice = GasPrice.fromString('0.0000025ustake');
      updateFee = calculateFee(2_000_000, gasPrice);
      break;
    }
    default: {
      console.log('Incorrect network parameter');
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
  const result = await client.execute(
    DendrETHWalletInfo.address,
    contractAddress,
    JSON.parse(updateData),
    updateFee,
    'Updating the Verifier',
  );
  console.log(result);
}

Update();
