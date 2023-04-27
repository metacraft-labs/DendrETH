import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';
import * as fs from 'fs';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import {
  appendJsonFile,
  getRootDir,
  sleep,
} from '../../libs/typescript/ts-utils/common-utils';
import { setUpCosmosTestnet } from '../../libs/typescript/cosmos-utils/testnet-setup';

import {
  compileVerifierContract,
  compileVerifierNimFileToWasm,
  compileVerifierParseDataTool,
} from '../../contracts/cosmos/verifier/lib/typescript/verifier-compile-contract-and-tools';

const exec = promisify(exec_);

describe('Rust verifier in cosmos', () => {
  let contractDirVerifier: string;
  let pathToVerifyUtils: string;
  let pathToKey: string;
  let updateFiles: string[];
  let DendrETHWalletInfo;

  let controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'http://localhost:26657';
  const gasPrice = GasPrice.fromString('0.0000025ustake');
  let client: SigningCosmWasmClient;
  let _contractAddress;
  let cosmos;
  let rootDir;
  beforeAll(async () => {
    rootDir = await getRootDir();
    contractDirVerifier = rootDir + `/contracts/cosmos/rust-verifier`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
    pathToKey = pathToVerifyUtils + `vkey.json`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);

    await compileVerifierContract();

    cosmos = await setUpCosmosTestnet(rpcEndpoint, signal);
    client = cosmos.client;
    DendrETHWalletInfo = cosmos.walletInfo;
  }, 360000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    console.info("Running 'Check Verifier after initialization' test");
    const expectedHeaderHash =
      '196,61,148,170,234,19,66,248,229,81,217,165,230,254,149,183,235,176,19,20,42,207,30,38,40,173,56,30,92,113,51,22';
    // Loading the contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/rust-verifier/artifacts/verifier.wasm`,
    );

    // Upload the contract
    const uploadFee = calculateFee(2_500_000, gasPrice);
    console.log('Rust-verifier wasm file: ', wasm);
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      uploadFee,
      'Upload Rust Verifier in Cosmos contract',
    );
    console.info(
      'Upload of Rust `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );

    // Instantiating the smart contract
    const instantiateFee = calculateFee(2_000_000, gasPrice);

    // Instantiate the contract with the contract specific message
    const instantiation = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      JSON.parse('this is a path!'),
      'My instance',
      instantiateFee,
      { memo: 'Create a Verifier in Cosmos instance.' },
    );
    // Gas Used

    console.info('Contract instantiated at: ', instantiation.contractAddress);
    _contractAddress = instantiation.contractAddress;

    // Query contract after Instantiation
    const queryResultAfterInitialization = await client.queryContractSmart(
      _contractAddress,
      {
        last_header_hash: {},
      },
    );
    const headerHash = queryResultAfterInitialization
      .toString()
      .replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);
  }, 300000);
});
