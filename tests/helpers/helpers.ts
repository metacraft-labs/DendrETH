import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import { getRootDir } from '../../libs/typescript/ts-utils/common-utils';

const exec = promisify(exec_);

export function replaceInTextProof(updateFile) {
  let t = 0;
  const result = updateFile.replace(/proof/g, match =>
    ++t === 1 ? 'update' : match,
  );
  return result;
}

export class gasUsed {
  description: string;
  gas: number;

  constructor(description: string, gas: number) {
    this.description = description;
    this.gas = gas;
  }
}

async function getDirs(
  protocol: 'cosmos' | 'eos',
  contract: 'verifier' | 'light-client',
) {
  const rootDir = await getRootDir();
  const contractDir = `${rootDir}/contracts/${protocol}/${contract}`;
  return { rootDir, contractDir };
}

export async function compileVerifierParseDataTool(
  protocol: 'cosmos' | 'eos',
  contract: 'verifier' | 'light-client',
) {
  const { rootDir } = await getDirs(protocol, contract);
  const toolDir = `${rootDir}/tests/helpers/verifier-parse-data-tool`;
  const binaryPath = `${toolDir}/build/verifier_parse_data`;
  const compileParseDataTool = `nim c -d:nimOldCaseObjects \
  -o:${binaryPath} \
  "${toolDir}/verifier_parse_data.nim" `;

  console.info(
    `Building 'verifier-parse-data' tool \n  ╰─➤ ${compileParseDataTool}`,
  );

  await exec(compileParseDataTool);
  return binaryPath;
}
