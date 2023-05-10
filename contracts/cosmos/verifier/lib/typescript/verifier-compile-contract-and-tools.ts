import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

import { compileNimFileToWasm } from '../../../../../libs/typescript/ts-utils/compile-nim-to-wasm';
import { getCosmosContractArtifacts } from '../../../../../libs/typescript/cosmos-utils/cosmos-utils';

const exec = promisify(exec_);

export async function compileVerifierNimFileToWasm() {
  const { contractDir } = await getCosmosContractArtifacts('verifier');
  const nimFilePath = contractDir + `/lib/nim//verify/verify.nim`;

  await compileNimFileToWasm(
    nimFilePath,
    `--nimcache:"${contractDir}"/nimcache --d:lightClientCosmos \
    -o:"${contractDir}/nimcache/nim_verifier.wasm"`,
  );
}

export async function compileVerifierParseDataTool() {
  const { rootDir, contractDir } = await getCosmosContractArtifacts('verifier');
  const compileParseDataTool = `nim c -d:nimOldCaseObjects -o:"${contractDir}/nimcache/" \
  "${rootDir}/tests/cosmosLightClient/helpers/verifier-parse-data-tool/verifier_parse_data.nim" `;

  console.info(
    `Building 'verifier-parse-data' tool \n  ╰─➤ ${compileParseDataTool}`,
  );

  await exec(compileParseDataTool);
}

export async function compileVerifierContract(patch: string | null) {
  const { contractDir, wasmContractPath } = await getCosmosContractArtifacts(
    'verifier',
  );

  let dockerPatch = '';
  if (patch !== null) {
    dockerPatch = `-v "${contractDir}-${patch}/Cargo.toml":/code/Cargo.toml \
                   -v "${contractDir}-${patch}/Cargo.lock":/code/Cargo.lock`;
  }

  const compileContractCommandVerify = `docker run -t --rm \
  -v "${contractDir}":/code \
  ${dockerPatch} \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.11 .`;

  console.info(`Building the contract \n  ╰─➤ ${compileContractCommandVerify}`);

  await exec(compileContractCommandVerify);
  console.info(`Compiling the Verifier contract finished \n`);
  console.info(`The wasm contract file is at:`, wasmContractPath);
  return wasmContractPath;
}

export async function compileContractMain(patch: string | null) {
  await compileVerifierNimFileToWasm();
  await compileVerifierParseDataTool();
  await compileVerifierContract(patch);
}
