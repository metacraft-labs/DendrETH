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
import { compileContractMain } from '../../contracts/cosmos/verifier/typescript/verifier-compile-contract-and-tools';
import {
  instantiateVerifierContract,
  uploadVerifierContract,
} from '../../contracts/cosmos/verifier/typescript/verifier-upload-instantiate';
import { updateVerifierContract } from '../../contracts/cosmos/verifier/typescript/verifier-make-update';
import { replaceInTextProof, gasUsed } from '../helpers/helpers';
import { getCosmosContractArtifacts } from '../../libs/typescript/cosmos-utils/cosmos-utils';

const exec = promisify(exec_);

describe('Light Client Verifier In Cosmos', () => {
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

  const gasUsageFile = 'tests/cosmosLightClient/gasVerifierConstantine.json';

  beforeAll(async () => {
    const { rootDir, contractDir } = await getCosmosContractArtifacts(
      'verifier-constantine',
    );
    parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates-94/`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    await compileContractMain(null, 'verifier-constantine');
    cosmos = await setUpCosmosTestnet(mnemonic, 'verifier-constantine');
    client = cosmos.client;
  }, 720000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    console.info("Running 'Check Verifier after initialization' test");
    const expectedHeaderHash =
      '76,231,107,116,120,203,14,238,74,50,199,242,91,181,97,202,29,15,68,77,23,22,200,246,242,96,144,14,244,95,55,210';
    const expectedDomain =
      '7,0,0,0,98,137,65,239,33,209,254,140,113,52,114,10,221,16,187,145,227,176,44,0,126,0,70,210,71,44,102,149';

    const uploadReceipt = await uploadVerifierContract(
      'wasm',
      cosmos,
      'verifier-constantine',
    );
    console.info(
      'Upload of `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );

    let uploadGas = new gasUsed('Upload Verifier', uploadReceipt.gasUsed);
    gasArrayVerifier.push(uploadGas);

    const defaultInitHeaderRoot =
      '0x4ce76b7478cb0eee4a32c7f25bb561ca1d0f444d1716c8f6f260900ef45f37d2';
    const defaultDomain =
      '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';
    const instantiation = await instantiateVerifierContract(
      uploadReceipt,
      defaultInitHeaderRoot,
      defaultDomain,
      cosmos,
      'verifier-constantine',
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

    // Query contract for domain
    const domain = await client.queryContractSmart(_contractAddress, {
      domain: {},
    });
    const domainStripped = domain.toString().replace(/\s/g, '');
    expect(domainStripped).toEqual(expectedDomain);
  }, 300000);

  test('Check "Verifier" after one update', async () => {
    console.info("Running 'Check Verifier after one update' test");

    var updatePath;
    for (var proofFilePath of updateFiles.slice(1, 2)) {
      updatePath = replaceInTextProof(proofFilePath);
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );
      const result = await updateVerifierContract(
        'wasm',
        cosmos,
        _contractAddress,
        updateNumber,
        'verifier-constantine',
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

    const numOfUpdates = 34;
    var updatePath;
    var updateCounter = 1;
    for (var proofFilePath of updateFiles.slice(2, numOfUpdates)) {
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
        'verifier-constantine',
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
    const headerHashAfter33Update = await client.queryContractSmart(
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
    const finalizedHeaderHashAfter33Update = await client.queryContractSmart(
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
    const execStateRootAfter33Update = await client.queryContractSmart(
      _contractAddress,
      {
        last_exec_state_root: {},
      },
    );

    const getExpectedSlotCommand = `${parseDataTool} expectedSlot --expectedSlot=${updatePath}`;
    console.info(`Parsing expected latest slot \n   ${getExpectedSlotCommand}`);
    const expectedSlotExec = exec(getExpectedSlotCommand);
    const expectedSlot = (await expectedSlotExec).stdout
      .toString()
      .replace(/\s/g, '')
      .replace('[', '')
      .replace(']', '');
    console.info(`Parsed expected latest slot: \n  ╰─➤ [${expectedSlot}]`);
    // Query contract for slot after 33 updates
    const SlotAfter33Update = await client.queryContractSmart(
      _contractAddress,
      {
        current_slot: {},
      },
    );

    const headerHash = headerHashAfter33Update.toString().replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);
    const finalizedHeader = finalizedHeaderHashAfter33Update
      .toString()
      .replace(/\s/g, '');
    expect(finalizedHeader).toEqual(expectedFinalizedHeaderHash);
    const execStateRoot = execStateRootAfter33Update
      .toString()
      .replace(/\s/g, '');
    expect(execStateRoot).toEqual(expectedExecStateRoot);
    expect(SlotAfter33Update.toString()).toEqual(expectedSlot);

    appendJsonFile(gasUsageFile, gasArrayVerifier);

    await stopCosmosNode('verifier-constantine');
  }, 2000000);
});
