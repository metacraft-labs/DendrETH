import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

import {
  setUpCosmosTestnet,
  stopCosmosNode,
} from '../../libs/typescript/cosmos-utils/testnet-setup';
import {
  appendJsonFile,
  sleep,
} from '../../libs/typescript/ts-utils/common-utils';
import { compileContractMain } from '../../contracts/cosmos/verifier/lib/typescript/verifier-compile-contract-and-tools';
import {
  instantiateVerifierContract,
  uploadVerifierContract,
} from '../../contracts/cosmos/verifier/lib/typescript/verifier-upload-instantiate';
import { updateVerifierContract } from '../../contracts/cosmos/verifier/lib/typescript/verifier-make-update';
import { replaceInTextProof, gasUsed } from '../helpers/helpers';
import { getCosmosContractArtifacts } from '../../libs/typescript/cosmos-utils/cosmos-utils';

const exec = promisify(exec_);

describe('Light Client Verifier In Cosmos', () => {
  let contractDirVerifier: string;
  let parseDataTool: string;
  let pathToVerifyUtils: string;
  let updateFiles: string[];
  let gasArrayVerifier: gasUsed[] = [];
  let client: SigningCosmWasmClient;
  let _contractAddress;
  let cosmos;

  let controller = new AbortController();
  const { signal } = controller;

  const mnemonic =
    'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone';

  const gasUsageFile = 'tests/cosmosLightClient/gasVerifier.json';

  beforeAll(async () => {
    const { rootDir, contractDir } = await getCosmosContractArtifacts(
      'verifier',
    );

    parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);

    await compileContractMain(null);

    cosmos = await setUpCosmosTestnet(mnemonic);
    client = cosmos.client;
  }, 360000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    console.info("Running 'Check Verifier after initialization' test");
    const expectedHeaderHash =
      '196,61,148,170,234,19,66,248,229,81,217,165,230,254,149,183,235,176,19,20,42,207,30,38,40,173,56,30,92,113,51,22';

    const uploadReceipt = await uploadVerifierContract('wasm', cosmos);
    console.info(
      'Upload of `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );

    let uploadGas = new gasUsed('Upload Verifier', uploadReceipt.gasUsed);
    gasArrayVerifier.push(uploadGas);

    const defaultInitHeaderRoot =
      '0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316';
    const instantiation = await instantiateVerifierContract(
      uploadReceipt,
      defaultInitHeaderRoot,
      cosmos,
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
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );
      const result = await updateVerifierContract(
        'wasm',
        cosmos,
        _contractAddress,
        updateNumber,
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
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );
      updateCounter++;

      const result = await updateVerifierContract(
        'wasm',
        cosmos,
        _contractAddress,
        updateNumber,
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

    await stopCosmosNode();
  }, 2000000);
});
