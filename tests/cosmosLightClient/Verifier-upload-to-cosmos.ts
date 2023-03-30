import { dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { readFile } from 'fs/promises';
import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { SSZSpecTypes } from '../../libs/typescript/ts-utils/sszSpecTypes';
import { jsonToSerializedBase64 } from '../../libs/typescript/ts-utils/ssz-utils';
import { compileNimFileToWasm } from '../../libs/typescript/ts-utils/compile-nim-to-wasm';
import { byteArrayToNumber } from '../../libs/typescript/ts-utils/common-utils';
import { stringify } from 'node:querystring';
import { Decimal } from '@cosmjs/math';

const exec = promisify(exec_);
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
function replaceInTextProof(updateFile) {
  let t = 0;
  const result = updateFile.replace(/proof/g, match =>
    ++t === 2 ? 'public' : match,
  );
  return result;
}
let rootDir;

describe('Light Client In Cosmos', () => {
  let contractDirVerifier: string;
  let verifierTool: string;
  let parseDataTool: string;
  let pathToVerifyUtils: string;
  let pathToKey: string;
  let pathToFirstHeader: string;
  const controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'https://explorer.public-testnet.fl.cudos.org:36657/';
  const gasPrice = GasPrice.fromString('0.0000025acudos');

  let DendrETHWalletInfo = {
    mnemonic:
      'eager idle rain salt gather ask hard note clinic ketchup badge bid',
    address: '',
  };
  class gasUsed {
    description: string;
    gas: number;

    constructor(description: string, gas: number) {
      this.description = description;
      this.gas = gas;
    }
  }
  let gasArrayVerifier: gasUsed[] = [];
  let wallet: OfflineSigner, client: SigningCosmWasmClient;
  let _contractAddress;
  beforeAll(async () => {
    rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
      /\s/g,
      '',
    );

    //Verifier
    contractDirVerifier = rootDir + `/contracts/cosmos/verifier`;
    verifierTool = `${contractDirVerifier}/nimcache/contractInteraction`;
    parseDataTool = `${contractDirVerifier}/nimcache/parseData`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
    pathToKey = pathToVerifyUtils + `vkey.json`;
    pathToFirstHeader = pathToVerifyUtils + `update_5200024_5200056.json`;

    let nimFilePathVerifier = contractDirVerifier + `/lib/nim/verify.nim`;
    await compileNimFileToWasm(
      nimFilePathVerifier,
      `--nimcache:"${contractDirVerifier}"/nimcache --d:lightClientCosmos \
        -o:"${contractDirVerifier}/nimcache/verifier.wasm"`,
    );

    let compileNimVerifierTool = `nim c -d:nimOldCaseObjects -o:"${contractDirVerifier}/nimcache/" \
     "${rootDir}/contracts/cosmos/verifier/lib/nim/contractInteraction.nim" `;
    console.info(`➤ ${compileNimVerifierTool}`);
    await exec(compileNimVerifierTool);

    let compileParseDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDirVerifier}/nimcache/" \
    "${rootDir}/tests/cosmosLightClient/helpers/parseData/parseData.nim" `;
    console.info(`➤ ${compileParseDataTool}`);
    await exec(compileParseDataTool);

    let compileContractCommandVerify = `docker run -t --rm -v "${contractDirVerifier}":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
          cosmwasm/rust-optimizer:0.12.12 .`;
    console.info(`➤ ${compileContractCommandVerify}`);
    await exec(compileContractCommandVerify);

    const setupWasmdCommand = `bash "${rootDir}/contracts/cosmos/scripts/setup_wasmd.sh"`;
    console.info(`➤ ${setupWasmdCommand}`);
    execSync(setupWasmdCommand);

    const startNodeCommand = `bash "${rootDir}/contracts/cosmos/scripts/start_node.sh"`;
    console.info(`➤ ${startNodeCommand}`);
    exec_(startNodeCommand, { signal });

    await sleep(15000);

    const addKeyCommand = `bash "${rootDir}/contracts/cosmos/scripts/add_account.sh"`;
    console.info(`➤ ${addKeyCommand}`);
    execSync(addKeyCommand);

    const getFredAddressCommand = `wasmd keys show fred -a --keyring-backend test \
      --keyring-dir $HOME/.wasmd_keys`;
    console.info(`➤ ${getFredAddressCommand}`);
    const getAddress = exec(getFredAddressCommand);
    const fredAddress = (await getAddress).stdout;
    DendrETHWalletInfo.address = 'cudos13k5ktkd6lzvegzwrx8nxmxu5u4pqj7z8tzfszm'; // fredAddress.trimEnd();
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
        gasPrice: GasPrice.fromString('10000000000000000000acudos'),
      },
    );
  }, 360000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    // The contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/verifier/artifacts/verifier.wasm`,
    );

    // Upload the contract
    const uploadFee = 7500000000000000000;
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      'auto',
      'Upload Cosmos Light Client contract',
    );
    console.info('Upload succeeded. Receipt:', uploadReceipt);
    let uploadGas = new gasUsed('Upload Verifier', uploadReceipt.gasUsed);

    gasArrayVerifier.push(uploadGas);

    // Instantiating the smart contract
    const instantiateFee = calculateFee(2_000_000, gasPrice);
    // Parse the contract specific message that is passed to the contract
    const parseInitDataCommand = `${parseDataTool} initData --initHeaderPath=${pathToFirstHeader} \
    --verificationKeyPath=${pathToKey}`;
    console.info(`➤ ${parseInitDataCommand}`);
    const updateDataExec = exec(parseInitDataCommand);
    const initData = (await updateDataExec).stdout.replace(/\s/g, '');

    // Instantiate the contract with the contract specific message
    const initer = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      JSON.parse(initData),
      'My instance',
      instantiateFee,
      { memo: 'Create a Cosmos Light Client instance.' },
    );
    // Gas Used
    console.info(`Init used ` + initer.gasUsed + ` gas`);
    let initGas = new gasUsed('Init Verifier', initer.gasUsed);
    gasArrayVerifier.push(initGas);

    console.info('Contract instantiated at: ', initer.contractAddress);
    _contractAddress = initer.contractAddress;

    //What is the expected result of the query below
    const getExpectedHeaderCommand =
      `${parseDataTool} currentHeader --currentHeaderPath=` + pathToFirstHeader;
    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = execSync(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec)
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');

    // Query contract after Instantiation
    const queryResultAfterInitialization = await client.queryContractSmart(
      _contractAddress,
      {
        header: {},
      },
    );

    const header = queryResultAfterInitialization.toString().replace(/\s/g, '');
    expect(header).toEqual(
      '196,61,148,170,234,19,66,248,229,81,217,165,230,254,149,183,235,176,19,20,42,207,30,38,40,173,56,30,92,113,51,22',
    );
  }, 300000);

  test('Check "Verifier" after one update', async () => {
    // Executing update on the smart contract
    const updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    var newHeaderPath;
    var counter = 1;
    for (var proofFilePath of updateFiles.slice(0, 1)) {
      console.log(proofFilePath);
      newHeaderPath = replaceInTextProof(proofFilePath);

      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateData \
        --proofPath=${proofFilePath} --numberOfUpdate=${newHeaderPath}`;
      console.info(`➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');

      // Execute update on the contract with the contract specific message
      const executeFee = calculateFee(2_000_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        JSON.parse(updateData),
        executeFee,
      );

      // Gas Used
      counter++;
      console.info(`Update ` + counter + ` used ` + result.gasUsed + ` gas`);
      let updateGas = new gasUsed(`Update ` + counter, result.gasUsed);
      gasArrayVerifier.push(updateGas);
    }

    //What is the expected result of the query below
    const getExpectedHeaderCommand = `${parseDataTool} newHeader --newHeaderPath=${newHeaderPath}`;

    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    await sleep(10000);

    // Query contract after one update
    const headerSlotAfterOneUpdate = await client.queryContractSmart(
      _contractAddress,
      {
        header: {},
      },
    );

    const header = headerSlotAfterOneUpdate.toString().replace(/\s/g, '');

    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after 20 updates', async () => {
    const updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    const numOfUpdates = 20;
    var newHeaderPath;
    var counter = 1;
    for (var proofFilePath of updateFiles.slice(1, numOfUpdates)) {
      const newHeaderPath = replaceInTextProof(proofFilePath);

      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateData \
        --proofPath=${proofFilePath} --numberOfUpdate=${newHeaderPath}`;
      console.info(`➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');

      // Execute update on the contract with the contract specific message
      const executeFee = calculateFee(2_000_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        JSON.parse(updateData),
        executeFee,
      );

      // Gas Used
      counter++;
      console.info(`Update ` + counter + ` used ` + result.gasUsed + ` gas`);
      let updateGas = new gasUsed(`Update ` + counter, result.gasUsed);
      gasArrayVerifier.push(updateGas);
    }

    //What is the expected result of the query below
    const getExpectedHeaderCommand = `${parseDataTool} newHeader --newHeaderPath=${newHeaderPath}`;
    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');

    // Query contract after 20 updates
    const headerSlotAfter20Update = await client.queryContractSmart(
      _contractAddress,
      {
        header: {},
      },
    );

    const header = headerSlotAfter20Update.toString().replace(/\s/g, '');
    fs.writeFileSync(
      'tests/cosmosLightClient/gasVerifier.json',
      JSON.stringify(gasArrayVerifier),
      {
        flag: 'w',
      },
    );
    expect(header).toEqual(expectedHeader);
    controller.abort();
  }, 1500000);
});
