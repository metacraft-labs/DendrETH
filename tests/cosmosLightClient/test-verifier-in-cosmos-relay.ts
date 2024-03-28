import { readFileSync } from 'fs';
import { beforeAll, describe, expect, test } from '@jest/globals';

import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import {
  setUpCosmosTestnet,
  stopCosmosNode,
} from '@dendreth/utils/cosmos-utils/testnet-setup';
import { CosmosContract } from '@dendreth/relay/implementations/cosmos-contract';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { getRootDir, sleep } from '@dendreth/utils/ts-utils/common-utils';
import { compileContractMain } from '../../contracts/cosmos/verifier/typescript/verifier-compile-contract-and-tools';
import {
  instantiateVerifierContract,
  uploadVerifierContract,
} from '../../contracts/cosmos/verifier/typescript/verifier-upload-instantiate';
import { replaceInTextProof } from '../helpers/helpers';

const exec = promisify(exec_);

describe('Light Client Verifier In Cosmos', () => {
  let contractDirVerifier: string;
  let parseDataTool: string;
  let pathToVerifyUtils: string;
  let updateFiles: string[];
  let cosmosContract: CosmosContract;
  let DendrETHWalletInfo;
  let cosmos;

  let controller = new AbortController();
  const { signal } = controller;

  const rpcEndpoint = 'http://localhost:26657';
  const mnemonic =
    'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone';

  beforeAll(async () => {
    const rootDir = await getRootDir();

    contractDirVerifier =
      rootDir + `/contracts/cosmos/verifier/verifier-bncurve`;
    parseDataTool = `${contractDirVerifier}/nimcache/verifier_parse_data`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates-94/`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);

    await compileContractMain(null, 'verifier-bncurve');

    cosmos = await setUpCosmosTestnet(mnemonic, 'verifier-bncurve');
    DendrETHWalletInfo = cosmos.walletInfo;
  }, 720000 /* timeout in milliseconds */);

  test('Check "Verifier" after initialization', async () => {
    console.info("Running 'Check Verifier after initialization' test");
    const expectedHeaderHash =
      '0x4ce76b7478cb0eee4a32c7f25bb561ca1d0f444d1716c8f6f260900ef45f37d2';
    const uploadReceipt = await uploadVerifierContract(
      'wasm',
      cosmos,
      'verifier-bncurve',
    );
    console.info(
      'Upload of `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );
    console.info(
      'Upload of `Verifier in Cosmos` succeeded. Receipt:',
      uploadReceipt,
    );

    const defaultInitHeaderRoot = JSON.parse(
      readFileSync(replaceInTextProof(updateFiles[0])),
    ).attestedHeaderRoot;

    const defaultDomain =
      '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';

    const instantiation = await instantiateVerifierContract(
      uploadReceipt,
      defaultInitHeaderRoot,
      defaultDomain,
      cosmos,
      'verifier-bncurve',
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
    for (var proofFilePath of updateFiles.slice(1, 2)) {
      updatePath = replaceInTextProof(proofFilePath);

      // Parse the contract specific message that is passed to the contract
      const parseUpdateDataCommand = `${parseDataTool} updateDataForRelayTest \
      --proofPathRelay=${proofFilePath} --updatePathRelay=${updatePath}`;
      console.info(`Parsing data for update 1: \n ➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info(`Parsed update data: \n  ╰─➤ ${updateData}`);
      // Execute update on the contract with the contract specific message
      await cosmosContract.postUpdateOnChain(JSON.parse(updateData));
    }

    //What is the expected result of the query below
    const getExpectedHeaderCommand = `${parseDataTool} expectedHeaderRootPath \
    --expectedHeaderRootPath=${updatePath}`;

    console.info(
      `Parsing expected new header \n  ╰─➤ ${getExpectedHeaderCommand}`,
    );
    const expectedHeaderExec = exec(getExpectedHeaderCommand);
    const expectedHeader =
      '0x' +
      bytesToHex(
        new Uint8Array(
          (await expectedHeaderExec).stdout
            .toString()
            .replace(/\s/g, '')
            .replace('[', '')
            .replace(']', '')
            .split(',')
            .map(Number),
        ),
      );

    console.info(`Parsed expected new header: \n  ╰─➤ [${expectedHeader}]`);
    // await sleep(10000);
    // Query contract after one update
    const headerAfterOneUpdate = await cosmosContract.optimisticHeaderRoot();

    const header = headerAfterOneUpdate.toString().replace(/\s/g, '');
    expect(header).toEqual(expectedHeader);
  }, 300000);

  test('Check "Verifier" after 5 updates', async () => {
    console.info("Running 'Check Verifier after 5 updates' test");

    const numOfUpdates = 6;
    var updatePath;
    var updateCounter = 1;
    for (var proofFilePath of updateFiles.slice(2, numOfUpdates)) {
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
      await cosmosContract.postUpdateOnChain(JSON.parse(updateData));
    }

    // What is the expected result of the query below
    const getExpectedHeaderHashCommand = `${parseDataTool} expectedHeaderRootPath --expectedHeaderRootPath=${updatePath}`;
    console.info(
      `Parsing expected latest optimistic header hash \n   ${getExpectedHeaderHashCommand}`,
    );
    const expectedHeaderHahsExec = exec(getExpectedHeaderHashCommand);
    const expectedHeaderHash =
      '0x' +
      bytesToHex(
        new Uint8Array(
          (await expectedHeaderHahsExec).stdout
            .toString()
            .replace(/\s/g, '')
            .replace('[', '')
            .replace(']', '')
            .split(',')
            .map(Number),
        ),
      );
    console.info(
      `Parsed expected latest optimistic header hash: \n  ╰─➤ [${expectedHeaderHash}]`,
    );

    const headerHashAfter20Update = await cosmosContract.optimisticHeaderRoot();
    const headerHash = headerHashAfter20Update.toString().replace(/\s/g, '');
    expect(headerHash).toEqual(expectedHeaderHash);

    await stopCosmosNode('verifier-bncurve');
  }, 2000000);
});
