import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';

import { compileNimFileToWasm } from '../../../libs/typescript/ts-utils/compile-nim-to-wasm';

const exec = promisify(exec_);
async function CompileMain() {
  var rootDir = (await exec('git rev-parse --show-toplevel')).stdout.replace(
    /\s/g,
    '',
  );
  const contractDir = rootDir + `/contracts/cosmos/verifier`;
  const nimFilePath = contractDir + `/lib/nim//verify/verify.nim`;

  await compileNimFileToWasm(
    nimFilePath,
    `--nimcache:"${contractDir}"/nimcache --d:lightClientCosmos \
    -o:"${contractDir}/nimcache/verifier.wasm"`,
  );

  let compileParseDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDir}/nimcache/" \
  "${rootDir}/tests/cosmosLightClient/helpers/verifier-parse-data-tool/verifier_parse_data.nim" `;

  console.info(
    `Building 'verifier-parse-data' tool \n  ╰─➤ ${compileParseDataTool}`,
  );

  await exec(compileParseDataTool);

  let compileContractCommandVerify = `docker run -t --rm -v "${contractDir}":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.8 .`;

  console.info(`Building the contract \n  ╰─➤ ${compileContractCommandVerify}`);

  await exec(compileContractCommandVerify);
  console.info(`Compiling the Verifier contract finished \n`);
  console.info(
    `The nimwasm file is at:`,
    '/contracts/cosmos/verifier/nimcache/verifier.wasm',
  );
}

CompileMain();
