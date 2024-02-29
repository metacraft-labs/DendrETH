import { beforeAll, describe, expect, test } from '@jest/globals';

import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import {
  compileVerifierParseDataTool,
  replaceInTextProof,
} from '../helpers/helpers';
import { getRootDir, sleep } from '../../libs/typescript/ts-utils/common-utils';
import { EOSContract } from '../../relay/implementations/eos-contract';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';

const exec = promisify(exec_);

describe('Verifier in EOS', () => {
  console.info('Verifier in EOS test');
  let updateFiles: string[];
  let pathToVerifyUtils: string;
  let parseDataTool: string;
  let pathToKey: string;
  let stopLocalNodeCommand: string;
  let eosContract: EOSContract;
  let eosAccName: string;
  let rpcEndpoint: string;

  eosAccName = 'dendreth';
  const verifierTableKey = eosAccName;
  rpcEndpoint = 'local';

  const defaultInitHeaderRoot =
    '0x4ce76b7478cb0eee4a32c7f25bb561ca1d0f444d1716c8f6f260900ef45f37d2';
  const defaultDomain =
    '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';
  beforeAll(async () => {
    const rootDir = await getRootDir();
    parseDataTool = await compileVerifierParseDataTool(
      'eos',
      'verifier',
      'verifier-bncurve',
    );
    // await compileVerifierParseDataTool();
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates-94/`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    pathToKey = pathToVerifyUtils + `vk.json`;
    stopLocalNodeCommand = `bash ${rootDir}/contracts/eos/scripts/run_eos_testnet.sh stop`;
    const startLocalNodeCommand = `bash ${rootDir}/contracts/eos/scripts/run_eos_testnet.sh`;
    const buildAndDeployContractCommand = `bash ${rootDir}/contracts/eos/verifier-native/scripts/build.sh \
    && bash ${rootDir}/contracts/eos/verifier-native/scripts/deploy.sh ${eosAccName}`;
    await exec(stopLocalNodeCommand);
    await exec(startLocalNodeCommand);
    await exec(buildAndDeployContractCommand);
    console.info('Running testnet');
  }, 300000);

  test('Check "Verifier" after initialization', async () => {
    console.info('Verifier initialization');

    const parseInitDataCommand = `${parseDataTool} initDataEOS \
    --initHeaderRootEOS=${defaultInitHeaderRoot} \
    --verificationKeyPathEOS=${pathToKey} \
    --domainEOS=${defaultDomain}`;
    console.info(
      `Parsing data for instantiation. \n  ╰─➤ ${parseInitDataCommand}`,
    );
    const initDataExec = exec(parseInitDataCommand);
    const initData = (await initDataExec).stdout.replace(/\s/g, '');
    console.info(`Parsed instantiation data: \n  ╰─➤ ${initData}`);
    const initCommand =
      'cleos push action dendreth instantiate ' +
      initData +
      ' -p dendreth@active ';
    console.info('initCommand:', initCommand);
    await exec(initCommand);
    eosContract = new EOSContract(eosAccName, rpcEndpoint);
    // Query contract after Instantiation
    const queryResultAfterInitialization =
      await eosContract.optimisticHeaderRoot();
    console.info('After init query:', queryResultAfterInitialization);

    expect(queryResultAfterInitialization).toEqual(
      '0x' +
        bytesToHex(
          new Uint8Array([
            76, 231, 107, 116, 120, 203, 14, 238, 74, 50, 199, 242, 91, 181, 97,
            202, 29, 15, 68, 77, 23, 22, 200, 246, 242, 96, 144, 14, 244, 95,
            55, 210,
          ]),
        ),
    );
  }, 30000);

  test('Check "Verifier" after 1 update', async () => {
    console.info('Verifier after 1 update');

    var updatePath;
    for (var proofFilePath of updateFiles.slice(1, 2)) {
      updatePath = replaceInTextProof(proofFilePath);
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );

      const parseUpdateDataCommand = `${parseDataTool} updateDataForRelayTest \
      --proofPathRelay=${proofFilePath} --updatePathRelay=${updatePath}`;
      console.info(`Parsing data for update: \n ➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info('updating with data:', updateData);

      await eosContract.postUpdateOnChain(JSON.parse(updateData));
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
    await sleep(10000);

    // Query contract after Instantiation
    const queryResultAfterUpdate = await eosContract.optimisticHeaderRoot();
    console.info('Result of query:', queryResultAfterUpdate);

    expect(queryResultAfterUpdate).toEqual(expectedHeader);
  }, 30000);

  test('Check "Verifier" after 33 update', async () => {
    console.info('Verifier after 33 update');

    var updatePath;
    var counter = 2;
    for (var proofFilePath of updateFiles.slice(2, 35)) {
      updatePath = replaceInTextProof(proofFilePath);
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );

      const parseUpdateDataCommand = `${parseDataTool} updateDataForRelayTest \
      --proofPathRelay=${proofFilePath} --updatePathRelay=${updatePath}`;
      console.info(
        `Parsing data for update ${counter}: \n  ╰─➤ ${parseUpdateDataCommand}`,
      );
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info('update ' + counter + ' with data:', updateData);

      await eosContract.postUpdateOnChain(JSON.parse(updateData));
      counter++;
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
    await sleep(10000);

    const queryResultAfterUpdates = await eosContract.optimisticHeaderRoot();
    console.info('Result of query:', queryResultAfterUpdates);

    expect(queryResultAfterUpdates).toEqual(expectedHeader);
    await exec(stopLocalNodeCommand);
  }, 120000);
});
