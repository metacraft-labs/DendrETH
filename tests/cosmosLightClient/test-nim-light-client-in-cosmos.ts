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
  let parseExpectedDataTool: string;
  let pathToVerifyUtils: string;
  let pathToKey: string;
  let pathToFirstHeader: string;
  const controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'http://localhost:26657';
  const gasPrice = GasPrice.fromString('0.0000025ustake');

  let DendrETHWalletInfo = {
    mnemonic:
      'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone',
    address: '',
  };

  let wallet: OfflineSigner, client: SigningCosmWasmClient;
  let _contractAddress;
  beforeAll(async () => {
    rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
      /\s/g,
      '',
    );

    // Light Client
    let contractDirLightClient = rootDir + `/contracts/cosmos/light-client`;
    let nimFilePathLightClient =
      contractDirLightClient + `/lib/nim/light_client_cosmos_wrapper.nim`;
    await compileNimFileToWasm(
      nimFilePathLightClient,
      `--nimcache:"${contractDirLightClient}"/nimbuild --d:lightClientCosmos -o:"${contractDirLightClient}"/nimbuild/light_client.wasm`,
    );
    let compileContractCommandLightClient = `docker run -t --rm -v "${contractDirLightClient}":/code --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/rust-optimizer:0.12.8 .`;
    console.info(`➤ ${compileContractCommandLightClient}`);

    await exec(compileContractCommandLightClient);

    //Verifier
    contractDirVerifier = rootDir + `/contracts/cosmos/verifier`;
    verifierTool = `${contractDirVerifier}/nimcache/contractInteraction`;
    parseExpectedDataTool = `${contractDirVerifier}/nimcache/parseExpectedData`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/mainnet/proofs/`;
    pathToKey = pathToVerifyUtils + `verification_key.json`;
    pathToFirstHeader = pathToVerifyUtils + `public291.json`;

    let nimFilePathVerifier = contractDirVerifier + `/lib/nim/verify.nim`;
    await compileNimFileToWasm(
      nimFilePathVerifier,
      `--nimcache:"${contractDirVerifier}"/nimcache --d:lightClientCosmos -o:"${contractDirVerifier}/nimcache/verifier.wasm"`,
    );

    let compileNimVerifierTool = `nim c -d:nimOldCaseObjects -o:"${contractDirVerifier}/nimcache/" "${rootDir}/contracts/cosmos/verifier/lib/nim/contractInteraction.nim" `;
    console.info(`➤ ${compileNimVerifierTool}`);
    await exec(compileNimVerifierTool);

    let compileParseExpectedDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDirVerifier}/nimcache/" "${rootDir}/tests/cosmosLightClient/helpers/parseExpectedData/parseExpectedData.nim" `;
    console.info(`➤ ${compileParseExpectedDataTool}`);
    await exec(compileParseExpectedDataTool);

    let compileContractCommandVerify = `docker run -t --rm -v "${contractDirVerifier}":/code --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/rust-optimizer:0.12.8 .`;
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

    const getFredAddressCommand = `wasmd keys show fred -a --keyring-backend test --keyring-dir $HOME/.wasmd_keys`;
    console.info(`➤ ${getFredAddressCommand}`);
    const getAddress = exec(getFredAddressCommand);
    const fredAddress = (await getAddress).stdout;
    DendrETHWalletInfo.address = fredAddress.trimEnd();
    wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      DendrETHWalletInfo.mnemonic,
      {
        prefix: 'wasm',
      },
    );
    client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet);
  }, 360000 /* timeout in milliseconds */);

  test('Check "LightClientStore" after initialization', async () => {
    const expectedHeaderSlot = 2375680;

    // The contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/light-client/artifacts/light_client.wasm`,
    );
    // Upload the contract
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      uploadFee,
      'Upload Cosmos Light Client contract',
    );
    console.info('Upload succeeded. Receipt:', uploadReceipt);

    // Instantiate the contract
    const instantiateFee = calculateFee(12_500_000, gasPrice);
    const bootstrapData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientBootstrap,
      rootDir + `/vendor/eth2-light-client-updates/mainnet/bootstrap.json`,
    );

    // This contract specific message is passed to the contract
    const msg = {
      bootstrap_data: bootstrapData,
    };
    const { contractAddress } = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      msg,
      'My instance',
      instantiateFee,
      { memo: 'Create a Cosmos Light Clinet instance.' },
    );
    console.info('Contract instantiated at: ', contractAddress);
    _contractAddress = contractAddress;

    // Query contract after initialization
    const queryResultAfterInitialization = await client.queryContractSmart(
      contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterInitialization = byteArrayToNumber(
      queryResultAfterInitialization.slice(0, 8),
    );

    expect(headerSlotAfterInitialization).toEqual(expectedHeaderSlot);
  }, 300000);

  test('Check "LightClientStore" after one update', async () => {
    const expectedHeaderSlot = 2381376;

    const updateData = await jsonToSerializedBase64(
      SSZSpecTypes.LightClientUpdate,
      rootDir + `/vendor/eth2-light-client-updates/mainnet/updates/00290.json`,
    );
    // This contract specific message is passed to the contract
    const execMsg = {
      update: {
        update_data: updateData,
      },
    };
    // Execute contract
    const executeFee = calculateFee(12_500_000, gasPrice);
    const result = await client.execute(
      DendrETHWalletInfo.address,
      _contractAddress,
      execMsg,
      executeFee,
    );
    const wasmEvent = result.logs[0].events.find(e => e.type === 'wasm');
    console.info(
      'The "wasm" event emitted by the contract execution:',
      wasmEvent,
    );
    // Query contract after execution ( after one update )
    const queryResultAfterOneUpdate = await client.queryContractSmart(
      _contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterOneUpdate = byteArrayToNumber(
      queryResultAfterOneUpdate.slice(0, 8),
    );

    expect(headerSlotAfterOneUpdate).toEqual(expectedHeaderSlot);
  }, 300000);

  test('Check "LightClientStore" after all updates', async () => {
    const expectedHeaderSlot = 4366496;

    const updateFiles = glob(
      rootDir + `/vendor/eth2-light-client-updates/mainnet/updates/*.json`,
    );
    for (var updateFile of updateFiles) {
      const updateData = await jsonToSerializedBase64(
        SSZSpecTypes.LightClientUpdate,
        updateFile,
      );
      const execMsg = {
        update: {
          update_data: updateData,
        },
      };
      // Execute contract
      const executeFee = calculateFee(12_500_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        execMsg,
        executeFee,
      );
      const wasmEvent = result.logs[0].events.find(e => e.type === 'wasm');
      console.info(
        'The "wasm" event emitted by the contract execution:',
        wasmEvent,
      );
    }

    // Query contract after execution ( after all updates )
    const queryResultAfterAllUpdates = await client.queryContractSmart(
      _contractAddress,
      {
        store: {},
      },
    );

    const headerSlotAfterAllUpdates = byteArrayToNumber(
      queryResultAfterAllUpdates.slice(0, 8),
    );

    expect(headerSlotAfterAllUpdates).toEqual(expectedHeaderSlot);
  }, 1500000);

  test('Check "Verifier" after initialization', async () => {
    // The contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/verifier/artifacts/verifier.wasm`,
    );

    // Upload the contract
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      uploadFee,
      'Upload Cosmos Light Client contract',
    );
    console.info('Upload succeeded. Receipt:', uploadReceipt);
    //Initializing the smart contract
    const getInitCommand =
      `${verifierTool}  --chain_id=testing --rpc=http://localhost:26657 init --code_id=` +
      uploadReceipt.codeId.toString() +
      ` --vKeyPath=` +
      pathToKey +
      ` --currentHeaderPath=` +
      pathToFirstHeader;
    console.info(`➤ ${getInitCommand}`);
    const initExec = exec(getInitCommand);
    _contractAddress = (await initExec).stdout;

    //What is the expected result of the query below
    const getExpectedHeaderCommand =
      `${parseExpectedDataTool} currentHeader --currentHeaderPath=` +
      pathToFirstHeader;
    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = execSync(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec)
      .toString()
      .replace(/\s/g, '');

    // Query contract after initialization
    const getQueryCommand =
      `${verifierTool} --chain_id=testing --rpc=http://localhost:26657 query --contract2=` +
      _contractAddress;
    console.info(`➤ ${getQueryCommand}`);
    const headerExec = execSync(getQueryCommand);
    const header = (await headerExec).toString().split(/:/)[1].split(/}/)[0];

    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after one update', async () => {
    //Update
    const pathToProof = pathToVerifyUtils + `proof291.json`;
    const getUpdateCommand =
      `${verifierTool}  --chain_id=testing --rpc=http://localhost:26657 update --proofPath=` +
      pathToProof +
      ` --newHeaderPath=` +
      pathToFirstHeader +
      ` --contract=` +
      _contractAddress;
    console.info(`➤ ${getUpdateCommand}`);
    exec(getUpdateCommand);

    //What is the expected result of the query below
    const getExpectedHeaderCommand =
      `${parseExpectedDataTool} newHeader --newHeaderPath=` + pathToFirstHeader;
    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec).stdout.replace(/\s/g, '');
    await sleep(10000);

    // Query contract after update
    const getQueryCommand =
      `${verifierTool} --chain_id=testing --rpc=http://localhost:26657 query --contract2=` +
      _contractAddress;
    console.info(`➤ ${getQueryCommand}`);
    const headerExec = exec(getQueryCommand);
    const header = (await headerExec).stdout.split(/:/)[1].split(/}/)[0];

    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after 20 updates', async () => {
    const updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    const numOfUpdates = 20;
    for (var updateFile of updateFiles.slice(1, numOfUpdates)) {
      const newHeaderPath = replaceInTextProof(updateFile);
      const getUpdateCommand =
        `${verifierTool}  update --proofPath=` +
        updateFile +
        ` --newHeaderPath=` +
        newHeaderPath +
        ` --contract=` +
        _contractAddress;
      console.info(`➤ ${getUpdateCommand}`);
      let response = await exec(getUpdateCommand);
      await sleep(5500);
    }

    //What is the expected result of the query below
    const getExpectedHeaderCommand =
      `${parseExpectedDataTool} newHeader --newHeaderPath=` +
      pathToVerifyUtils +
      `public` +
      (290 + numOfUpdates) +
      `.json`;
    console.info(`➤ ${getExpectedHeaderCommand}`);
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec).stdout.replace(/\s/g, '');

    // Query contract after updates
    const getQueryCommand =
      `${verifierTool} --chain_id=testing --rpc=http://localhost:26657 query --contract2=` +
      _contractAddress;
    console.info(`➤ ${getQueryCommand}`);
    const headerExec = exec(getQueryCommand);
    const header = (await headerExec).stdout.split(/:/)[1].split(/}/)[0];

    expect(header).toEqual(expectedHeader);
    controller.abort();
  }, 2000000);
});
