import glob_ from 'glob';
const glob = glob_.sync;
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { replaceInTextProof } from '../helpers/helpers';
import { sleep } from '../../libs/typescript/ts-utils/common-utils';
import { getRootDir } from '../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

describe('Verifier in EOS', () => {
  console.info('Verifier in EOS test');
  var rootDir: string;
  var contractDir: string;
  let updateFiles: string[];
  var pathToVerifyUtils: string;
  var parseDataTool: string;
  beforeAll(async () => {
    rootDir = await getRootDir();
    contractDir = rootDir + `/contracts/cosmos/verifier`;
    pathToVerifyUtils =
      rootDir + `/vendor/eth2-light-client-updates/prater/capella-updates/`;
    parseDataTool = `${contractDir}/nimcache/verifier_parse_data`;
    updateFiles = glob(pathToVerifyUtils + `proof*.json`);

    const runTestnetCommand =
      'bash contracts/eos/scripts/run_eos_testnet.sh stop && bash contracts/eos/scripts/run_eos_testnet.sh && bash contracts/eos/verifier/scripts/build.sh && bash contracts/eos/verifier/scripts/deploy.sh';
    await exec(runTestnetCommand);
    console.info('Running testnet');
  }, 300000);
  test('Check "Verifier" after initialization', async () => {
    console.info('Verifier initiolization');
    const initCommand =
      'cleos push action dendreth instantiate \'{"key":"dendreth", "verification_key":"170e112ab9a4cd01c36bab47402efccfe9ee4b1ae111de3ccf5e5c0f9802eb061e8b0ed6df2c4b3136b0295a1742e43c78027ecbaa357f1192653b4eda5146069d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0ee02767004aa368d40c5004b499c12979687767a7fe8104f57e1e73065a76f32a583e66c9641cd2331b9feb0e692900c8bf6a17d9e6244072bf850d012f90ca29369114eba7836e8fcd106037ef87fc7224fa34f8c712c6a4dae153a1a98a75153448f8a97a0d2418a87588d3fc40b6b9f9ca5b238c59592df4c6ccab03e7c22a9d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0e00000000000000000000000000000000000000000000000000000000000000002620bc02d1b5838e72017b493519ebdcdf1a81974726b8fb3b5096af4138571940614ca87d73b4afc4d802585add4360862fa052fc50e9096b7bea3a83f0fe14f6e96b889dfa9d61789b9ef597d27ffefe7d1b23621a9eff06429eaeeb7efd28ee5618c7565b0964bb3c7d3222f957dc76103533be35f9558264fd93e6a0a40d9d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0e00000000000000000000000000000000000000000000000000000000000000002620bc02d1b5838e72017b493519ebdcdf1a81974726b8fb3b5096af4138571940614ca87d73b4afc4d802585add4360862fa052fc50e9096b7bea3a83f0fe14f6e96b889dfa9d61789b9ef597d27ffefe7d1b23621a9eff06429eaeeb7efd28ee5618c7565b0964bb3c7d3222f957dc76103533be35f9558264fd93e6a0a40d9d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0e0000000000000000000000000000000000000000000000000000000000000000d469bf0e661c41864114449182ea8ff64598370fd114b8316c7a52e13623e91f029d5b05843499aa211c54b9eb355affdc33395847f742d1aac5d9cf2cbd1c309d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0e48b76a8fd9b89c5879252d81110920cf43808adec2d69c6ef6090adcc5ad3c226040d5a53a244a8936267f9ae7e95418d8622a662c55fc5a970a695ec4495b1f9d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0edde522c630999663eba20456e17e89c584aab9ac59568d1d4eef3de138ae521e911454787684589f06bcd042249e08cfb2f898742a1c2629fdd49cca1bf4eb2f9d0d8fc58d435dd33d0bc7f528eb780a2c4679786fa36e662fdf079ac1770a0e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "current_header_hash":"c43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316"}\' -p dendreth@active ';
    await exec(initCommand);
    const queryCommand =
      'cleos push action dendreth printheader "[]" -p dendreth@active';
    const queryRes = await exec(queryCommand);
    var result = (await queryRes).stdout.replace(/\s/g, '').substring(36);

    expect(result).toEqual(
      '[196,61,148,170,234,19,66,248,229,81,217,165,230,254,149,183,235,176,19,20,42,207,30,38,40,173,56,30,92,113,51,22]',
    );
  }, 30000);

  test('Check "Verifier" after 1 update', async () => {
    console.info('Verifier after 1 update');

    var updatePath;
    for (var proofFilePath of updateFiles.slice(0, 1)) {
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
    for (var proofFilePath of updateFiles.slice(1, 33)) {
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
      console.info('update' + counter + 'with data:', updateData);
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
  }, 30000);
});
