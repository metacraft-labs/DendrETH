import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { compileNimFileToWasm } from '../../../../../libs/typescript/ts-utils/compile-nim-to-wasm';
import { getRootDir } from '../../../../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

export async function compileVerifierNimFileToWasm() {
  const rootDir = await getRootDir();
  const contractDir = rootDir + `/contracts/cosmos/verifier`;
  const nimFilePath = contractDir + `/lib/nim//verify/verify.nim`;

  await compileNimFileToWasm(
    nimFilePath,
    `--nimcache:"${contractDir}"/nimcache --d:lightClientCosmos \
    -o:"${contractDir}/nimcache/nim_verifier.wasm"`,
  );
}

export async function compileVerifierParseDataTool() {
  const rootDir = await getRootDir();
  const contractDir = rootDir + `/contracts/cosmos/verifier`;
  const compileParseDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDir}/nimcache/" \
  "${rootDir}/tests/cosmosLightClient/helpers/verifier-parse-data-tool/verifier_parse_data.nim" `;

  console.info(
    `Building 'verifier-parse-data' tool \n  ╰─➤ ${compileParseDataTool}`,
  );

  await exec(compileParseDataTool);
}

export async function compileVerifierContract() {
  const rootDir = await getRootDir();
  const contractDir = rootDir + `/contracts/cosmos/verifier`;
  const compileContractCommandVerify = `docker run -t --rm -v "${contractDir}":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.8 .`;

  console.info(`Building the contract \n  ╰─➤ ${compileContractCommandVerify}`);

  await exec(compileContractCommandVerify);
  console.info(`Compiling the Verifier contract finished \n`);
  const contractPath = contractDir + '/artifacts/verifier.wasm';
  console.info(`The wasm contract file is at:`, contractPath);
  return contractPath;
}
