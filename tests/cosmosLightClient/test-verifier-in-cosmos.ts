import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { calculateFee, GasPrice } from '@cosmjs/stargate';
import * as fs from 'fs';

import { compileNimFileToWasm } from '../../libs/typescript/ts-utils/compile-nim-to-wasm';
import { setUpCosmosTestnet } from './helpers/testnet-setup';
import { appendJsonFile } from '../../libs/typescript/ts-utils/common-utils';

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

  let DendrETHWalletInfo;

  let controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'http://localhost:26657';
  const gasPrice = GasPrice.fromString('0.0000025ustake');

  class gasUsed {
    description: string;
    gas: number;

    constructor(description: string, gas: number) {
      this.description = description;
      this.gas = gas;
    }
  }
  let gasArrayVerifier: gasUsed[] = [];
  let client: SigningCosmWasmClient;
  let _contractAddress;

  const gasUsageFile = 'tests/cosmosLightClient/gasVerifier.json';

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

    let uploadGas = new gasUsed('Upload Verifier', uploadReceipt.gasUsed);
    gasArrayVerifier.push(uploadGas);

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
    // Gas Used
    console.info(`Instantiation used ` + instantiation.gasUsed + ` gas`);
    let initGas = new gasUsed('Init Verifier', instantiation.gasUsed);
    gasArrayVerifier.push(initGas);

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

  test('Check "Verifier" after one update', async () => {
    console.info("Running 'Check Verifier after one update' test");

    var updatePath;
    for (var proofFilePath of updateFiles.slice(0, 1)) {
      updatePath = replaceInTextProof(proofFilePath);

      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateData \
      --proofPath=${proofFilePath} --updatePath=${updatePath}`;
      console.info(`Parsing data for update 1: \n ➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);

      // Execute update on the contract with the contract specific message
      const executeFee = calculateFee(2_000_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        JSON.parse(updateData),
        executeFee,
      );

      // Gas Used logger
      console.info(`Update ` + 1 + ` used ` + result.gasUsed + ` gas`);
      let updateGas = new gasUsed(`Update ` + 1, result.gasUsed);
      gasArrayVerifier.push(updateGas);
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
    await sleep(10000);

    // Query contract after one update
    const headerAfterOneUpdate = await client.queryContractSmart(
      _contractAddress,
      {
        last_header_hash: {},
      },
    );

    const header = headerAfterOneUpdate.toString().replace(/\s/g, '');
    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after 33 updates', async () => {
    console.info("Running 'Check Verifier after 33 updates' test");

    const numOfUpdates = 33;
    var updatePath;
    var updateCounter = 1;
    for (var proofFilePath of updateFiles.slice(1, numOfUpdates)) {
      updatePath = replaceInTextProof(proofFilePath);

      updateCounter++;
      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateData \
        --proofPath=${proofFilePath} --updatePath=${updatePath}`;
      console.info(
        `Parsing data for update ${updateCounter}: \n  ╰─➤ ${parseUpdateDataCommand}`,
      );
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);

      // Execute update on the contract with the contract specific message
      const executeFee = calculateFee(2_000_000, gasPrice);
      const result = await client.execute(
        DendrETHWalletInfo.address,
        _contractAddress,
        JSON.parse(updateData),
        executeFee,
      );

      // Gas Used
      console.info(
        `Update ` + updateCounter + ` used ` + result.gasUsed + ` gas`,
      );
      let updateGas = new gasUsed(`Update ` + updateCounter, result.gasUsed);
      gasArrayVerifier.push(updateGas);
    }

    var currentUpdateNum = 1;
    //Query for optimistic_header_hash_array
    const allHeaderHashesOrdered = await client.queryContractSmart(
      _contractAddress,
      {
        all_header_hashes_ordered: {},
      },
    );
    //Query for finalized_header_hash_array
    const allFinalizedHeaderHashesOrdered = await client.queryContractSmart(
      _contractAddress,
      {
        all_finalized_header_hashes_ordered: {},
      },
    );
    //Query for execution_state_root_array
    const allExecStateRootsOrdered = await client.queryContractSmart(
      _contractAddress,
      {
        all_exec_state_roots_ordered: {},
      },
    );
    //Check if the 3 arrays on the smart contract are correctly filled
    for (var proofFilePath of updateFiles.slice(1, numOfUpdates)) {
      updatePath = replaceInTextProof(proofFilePath);
      currentUpdateNum++;
      if (numOfUpdates - currentUpdateNum < 32) {
        let num = numOfUpdates - currentUpdateNum;
        //Check if optimistic_header_hash_array on the smart contract is full with correct values
        const getExpectedHeaderHashCommand = `${parseDataTool} expectedHeaderRootPath --expectedHeaderRootPath=${updatePath}`;
        const expectedHeaderHashExec = exec(getExpectedHeaderHashCommand);
        const expectedHeaderHash = (await expectedHeaderHashExec).stdout
          .toString()
          .replace(/\s/g, '')
          .replace('[', '')
          .replace(']', '');
        const headerHash = allHeaderHashesOrdered[num]
          .toString()
          .replace(/\s/g, '');

        expect(headerHash).toEqual(expectedHeaderHash);

        //Check if finalized_header_hash_array on the smart contract is full with correct values
        const getExpectedFinalizedHeaderHashCommand = `${parseDataTool} expectedFinalizedRootPath --expectedFinalizedRootPath=${updatePath}`;
        const expectedFinalizedHeaderHashExec = exec(
          getExpectedFinalizedHeaderHashCommand,
        );
        const expectedFinalizedHeaderHash = (
          await expectedFinalizedHeaderHashExec
        ).stdout
          .toString()
          .replace(/\s/g, '')
          .replace('[', '')
          .replace(']', '');
        const finalizedHeaderHash = allFinalizedHeaderHashesOrdered[num]
          .toString()
          .replace(/\s/g, '');

        expect(finalizedHeaderHash).toEqual(expectedFinalizedHeaderHash);

        //Check if execution_state_root_array on the smart contract is full with correct values
        const getExpectedExecStateRootCommand = `${parseDataTool} expectedExecutionStateRoot --expectedExecutionStateRoot=${updatePath}`;
        const expectedExecStateRootExec = exec(getExpectedExecStateRootCommand);
        const expectedExecStateRoot = (await expectedExecStateRootExec).stdout
          .toString()
          .replace(/\s/g, '')
          .replace('[', '')
          .replace(']', '');
        const execStateRoot = allExecStateRootsOrdered[num]
          .toString()
          .replace(/\s/g, '');

        expect(execStateRoot).toEqual(expectedExecStateRoot);
      }
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
    // Query contract for optimisticHeaderHash after 33 updates
    const headerHashAfter20Update = await client.queryContractSmart(
      _contractAddress,
      {
        last_header_hash: {},
      },
    );

    const getExpectedFinalizedHeaderHashCommand = `${parseDataTool} expectedFinalizedRootPath --expectedFinalizedRootPath=${updatePath}`;
    console.info(
      `Parsing expected latest finalized header hash \n   ${getExpectedFinalizedHeaderHashCommand}`,
    );
    const expectedFinalizedHeaderHashExec = exec(
      getExpectedFinalizedHeaderHashCommand,
    );
    const expectedFinalizedHeaderHash = (
      await expectedFinalizedHeaderHashExec
    ).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    console.info(
      `Parsed expected latest finalized header hash: \n  ╰─➤ [${expectedFinalizedHeaderHash}]`,
    );
    // Query contract for finalizedHeaderHash after 33 updates
    const finalizedHeaderHashAfter20Update = await client.queryContractSmart(
      _contractAddress,
      {
        last_finalized_header_hash: {},
      },
    );

    const getExpectedExecStateRootCommand = `${parseDataTool} expectedExecutionStateRoot --expectedExecutionStateRoot=${updatePath}`;
    console.info(
      `Parsing expected latest exec state root \n   ${getExpectedExecStateRootCommand}`,
    );
    const expectedExecStateRootExec = exec(getExpectedExecStateRootCommand);
    const expectedExecStateRoot = (await expectedExecStateRootExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    console.info(
      `Parsed expected latest exec state root: \n  ╰─➤ [${expectedExecStateRoot}]`,
    );
    // Query contract for execStateRoot after 33 updates
    const execStateRootAfter20Update = await client.queryContractSmart(
      _contractAddress,
      {
        last_exec_state_root: {},
      },
    );

    const headerHash = headerHashAfter20Update.toString().replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);
    const finalizedHeader = finalizedHeaderHashAfter20Update
      .toString()
      .replace(/\s/g, '');
    expect(finalizedHeader).toEqual(expectedFinalizedHeaderHash);
    const execStateRoot = execStateRootAfter20Update
      .toString()
      .replace(/\s/g, '');
    expect(execStateRoot).toEqual(expectedExecStateRoot);

    appendJsonFile(gasUsageFile, gasArrayVerifier);

    controller.abort();
  }, 2000000);
});
