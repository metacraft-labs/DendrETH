import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import {
  compileVerifierParseDataTool,
  replaceInTextProof,
} from '../helpers/helpers';
import { getRootDir, sleep } from '../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

describe('Verifier in EOS', () => {
  console.info('Verifier in EOS test');
  let updateFiles: string[];
  let pathToVerifyUtils: string;
  let parseDataTool: string;
  let pathToKey: string;
  let stopLocalNodeCommand: string;
  const defaultInitHeaderRoot =
    '0x4ce76b7478cb0eee4a32c7f25bb561ca1d0f444d1716c8f6f260900ef45f37d2';
  const defaultDomain =
    '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';
  beforeAll(async () => {
    const rootDir = await getRootDir();
    parseDataTool = await compileVerifierParseDataTool('eos', 'verifier');
    // await compileVerifierParseDataTool();
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates-94/`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);
    pathToKey = pathToVerifyUtils + `vk.json`;
    stopLocalNodeCommand = `bash ${rootDir}/contracts/eos/scripts/run_eos_testnet.sh stop`;
    const startLocalNodeCommand = `bash ${rootDir}/contracts/eos/scripts/run_eos_testnet.sh`;
    const buildAndDeployContractCommand = `bash ${rootDir}/contracts/eos/verifier/scripts/build.sh \
    && bash ${rootDir}/contracts/eos/verifier/scripts/deploy.sh`;
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
    const queryCommand =
      'cleos push action dendreth printheader "[]" -p dendreth@active';
    const queryRes = await exec(queryCommand);
    var result = (await queryRes).stdout.replace(/\s/g, '').substring(36);
    console.info('After init query:', result);

    expect(result).toEqual(
      '[76,231,107,116,120,203,14,238,74,50,199,242,91,181,97,202,29,15,68,77,23,22,200,246,242,96,144,14,244,95,55,210]',
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

      const parseUpdateDataCommand = `${parseDataTool} updateDataEOS \
      --proofPathEOS=${pathToVerifyUtils}proof_${updateNumber} \
      --updatePathEOS=${pathToVerifyUtils}update_${updateNumber}`;
      console.info(`Parsing data for update: \n ➤ ${parseUpdateDataCommand}`);
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info('updating with data:', updateData);
      const updateCommand =
        'cleos push action dendreth update ' +
        updateData +
        ' -p dendreth@active';
      console.info('updateCommand:', updateCommand);

      await exec(updateCommand);
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
      .replace(/\s/g, '');
    console.info(`Parsed expected new header: \n  ╰─➤ [${expectedHeader}]`);
    await sleep(10000);

    const queryCommand =
      'cleos push action dendreth printheader "[]" -p dendreth@active';
    const queryRes = await exec(queryCommand);
    var result = (await queryRes).stdout.replace(/\s/g, '').substring(36);
    console.info('Result of query:', result);

    expect(result).toEqual(expectedHeader);
  }, 30000);

  test('Check "Verifier" after 33 update', async () => {
    console.info('Verifier after 33 update');

    var updatePath;
    var counter = 2;
    for (var proofFilePath of updateFiles.slice(2, 34)) {
      updatePath = replaceInTextProof(proofFilePath);
      const updateNumber = updatePath.substring(
        updatePath.indexOf('update_') + 7,
      );

      const parseUpdateDataCommand = `${parseDataTool} updateDataEOS \
      --proofPathEOS=${pathToVerifyUtils}proof_${updateNumber} \
      --updatePathEOS=${pathToVerifyUtils}update_${updateNumber}`;
      console.info(
        `Parsing data for update ` +
          counter +
          ` : \n ➤ ${parseUpdateDataCommand}`,
      );
      const updateDataExec = exec(parseUpdateDataCommand);
      const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
      console.info('update ' + counter + ' with data:', updateData);
      const updateCommand =
        'cleos push action dendreth update ' +
        updateData +
        ' -p dendreth@active';
      await exec(updateCommand);
      counter++;
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
      .replace(/\s/g, '');
    console.info(`Parsed expected new header: \n  ╰─➤ [${expectedHeader}]`);
    await sleep(10000);

    const queryCommand =
      'cleos push action dendreth printheader "[]" -p dendreth@active';
    const queryRes = await exec(queryCommand);
    var result = (await queryRes).stdout.replace(/\s/g, '').substring(36);
    console.info('Result of query:', result);

    const queryCommandall =
      'cleos push action dendreth printheaders "[]" -p dendreth@active';
    const queryResall = await exec(queryCommandall);
    var resultall = (await queryResall).stdout.replace(/\s/g, '').substring(37);
    console.info('Result of full query:', resultall);

    expect(result).toEqual(expectedHeader);
    await exec(stopLocalNodeCommand);
  }, 30000);
});
