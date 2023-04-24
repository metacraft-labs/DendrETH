import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { compileNimFileToWasm } from '../../libs/typescript/ts-utils/compile-nim-to-wasm';
import { setUpCosmosTestnet } from './helpers/testnet-setup';
import { CosmosContract } from '../../relay/implementations/cosmos-contract';

const exec = promisify(exec_);
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
function replaceInTextProof(updateFile) {
  let t = 0;
  const result = updateFile.replace(/proof/g, match =>
    ++t === 1 ? 'update' : match,
  );
  return result;
}
let rootDir;
describe('Light Client Verifier In Cosmos', () => {
  let contractDirVerifier: string;
  let parseDataTool: string;
  let pathToVerifyUtils: string;
  let pathToKey: string;
  let pathToFirstHeader: string;
  let updateFiles: string[];
  let cosmosContract: CosmosContract;

  let DendrETHWalletInfo;

  let controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'http://localhost:26657';
  const gasPrice = GasPrice.fromString('0.0000025ustake');

  let client: SigningCosmWasmClient;
  let _contractAddress;

  beforeAll(async () => {
    rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
      /\s/g,
      '',
    );

    contractDirVerifier = rootDir + `/contracts/cosmos/verifier`;
    parseDataTool = `${contractDirVerifier}/nimcache/verifier_parse_data`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
    pathToKey = pathToVerifyUtils + `vkey.json`;
    pathToFirstHeader = pathToVerifyUtils + `update_5200024_5200056.json`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);

    let nimFilePathVerifier =
      contractDirVerifier + `/lib/nim//verify/verify.nim`;
    await compileNimFileToWasm(
      nimFilePathVerifier,
      `--nimcache:"${contractDirVerifier}"/nimcache --d:lightClientCosmos \
      -o:"${contractDirVerifier}/nimcache/verifier.wasm"`,
    );

    let compileParseDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDirVerifier}/nimcache/" \
    "${rootDir}/tests/cosmosLightClient/helpers/verifier-parse-data-tool/verifier_parse_data.nim" `;
    console.info(
      `Building 'verifier-parse-data' tool \n  ╰─➤ ${compileParseDataTool}`,
    );
    await exec(compileParseDataTool);

    let compileContractCommandVerify = `docker run -t --rm -v "${contractDirVerifier}":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/rust-optimizer:0.12.8 .`;
    console.info(
      `Building the contract \n  ╰─➤ ${compileContractCommandVerify}`,
    );
    await exec(compileContractCommandVerify);

    let cosmos = await setUpCosmosTestnet(rootDir, rpcEndpoint, signal);
    client = cosmos.client;
    DendrETHWalletInfo = cosmos.DendrETHWalletInfo;
  }, 360000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    console.info("Running 'Check Verifier after initialization' test");
    const expectedHeaderHash =
      '196,61,148,170,234,19,66,248,229,81,217,165,230,254,149,183,235,176,19,20,42,207,30,38,40,173,56,30,92,113,51,22';
    // Loading the contract
    const wasm = fs.readFileSync(
      rootDir + `/contracts/cosmos/verifier/artifacts/verifier.wasm`,
    );

    // Upload the contract
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(
      DendrETHWalletInfo.address,
      wasm,
      uploadFee,
      'Upload Verifier in Cosmos contract',
    );
    console.info(
      'Upload of `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );

    // Instantiating the smart contract
    const instantiateFee = calculateFee(2_000_000, gasPrice);
    // Parse the contract specific message that is passed to the contract
    const parseInitDataCommand = `${parseDataTool} initData \
      --initHeaderPath=${pathToFirstHeader} \
      --verificationKeyPath=${pathToKey}`;
    console.info(
      `Parsing data for instantiation. \n  ╰─➤ ${parseInitDataCommand}`,
    );
    const updateDataExec = exec(parseInitDataCommand);
    const initData = (await updateDataExec).stdout.replace(/\s/g, '');
    console.info(`Parsed instantiation data: \n  ╰─➤ ${initData}`);

    // Instantiate the contract with the contract specific message
    const instantiation = await client.instantiate(
      DendrETHWalletInfo.address,
      uploadReceipt.codeId,
      JSON.parse(initData),
      'My instance',
      instantiateFee,
      { memo: 'Create a Verifier in Cosmos instance.' },
    );
    cosmosContract = new CosmosContract(
      instantiation.contractAddress,
      DendrETHWalletInfo.address,
      rpcEndpoint,
      'local',
    );
    console.info('Contract instantiated at: ', instantiation.contractAddress);

    // Query contract after Instantiation
    const queryResultAfterInitialization =
      await cosmosContract.optimisticHeaderRoot();

    const headerHash = queryResultAfterInitialization
      .toString()
      .replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);
  }, 300000);

  test('Check "Verifier" after one update', async () => {
    console.info("Running 'Check Verifier after one update' test");

    var updatePath;
    for (var proofFilePath of updateFiles.slice(0, 1)) {
      updatePath = replaceInTextProof(proofFilePath);

      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateDataForRelayTest \
      --proofPathRelay=${proofFilePath} --updatePathRelay=${updatePath}`;
      console.info(`Parsing data for update 1: \n ➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);
      // Execute update on the contract with the contract specific message
      cosmosContract.postUpdateOnChain(JSON.parse(updateData));
      await sleep(10000);
    }

    //What is the expected result of the query below
    const getExpectedHeaderCommand = `${parseDataTool} expectedHeaderRootPath \
    --expectedHeaderRootPath=${updatePath}`;

    console.info(
      `Parsing expected new header \n  ╰─➤ ${getExpectedHeaderCommand}`,
    );
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader = (await expectedHeaderExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    console.info(`Parsed expected new header: \n  ╰─➤ [${expectedHeader}]`);
    // await sleep(10000);
    // Query contract after one update
    const headerAfterOneUpdate = await cosmosContract.optimisticHeaderRoot();

    const header = headerAfterOneUpdate.toString().replace(/\s/g, '');
    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after 5 updates', async () => {
    console.info("Running 'Check Verifier after 5 updates' test");

    const numOfUpdates = 5;
    var updatePath;
    var updateCounter = 1;
    for (var proofFilePath of updateFiles.slice(1, numOfUpdates)) {
      updatePath = replaceInTextProof(proofFilePath);

      updateCounter++;
      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateDataForRelayTest \
        --proofPathRelay=${proofFilePath} --updatePathRelay=${updatePath}`;
      console.info(
        `Parsing data for update ${updateCounter}: \n  ╰─➤ ${parseUpdateDataCommand}`,
      );
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);

      // Execute update on the contract with the contract specific message
      cosmosContract.postUpdateOnChain(JSON.parse(updateData));
      await sleep(10000);
    }

    // What is the expected result of the query below
    const getExpectedHeaderHashCommand = `${parseDataTool} expectedHeaderRootPath --expectedHeaderRootPath=${updatePath}`;
    console.info(
      `Parsing expected latest optimistic header hash \n   ${getExpectedHeaderHashCommand}`,
    );
    const expectedHeaderHahsExec = exec(getExpectedHeaderHashCommand);
    const expectedHeaderHash = (await expectedHeaderHahsExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    console.info(
      `Parsed expected latest optimistic header hash: \n  ╰─➤ [${expectedHeaderHash}]`,
    );

    const headerHashAfter20Update = await cosmosContract.optimisticHeaderRoot();
    const headerHash = headerHashAfter20Update.toString().replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);

    controller.abort();
  }, 2000000);
});
