import { promisify } from 'node:util';
import { rm } from 'fs/promises';
import { resolve } from 'path';
import { exec as exec_ } from 'node:child_process';

const exec = promisify(exec_);

export async function compileNimFileToWasm(nimSourceFilepath: string) {
  const inputFileName = resolve(nimSourceFilepath);
  const outputFileName = inputFileName.replace(/\.nim$/, '.wasm');
  console.info(`➤ rm ${outputFileName}`);
  await rm(outputFileName, { force: true });
  const useEmcc = process.env.USE_EMCC === '1' ? '-d:emcc' : '';
  const command = `nim-wasm c --lib:"./vendor/nim/lib" --d:lightClientWASM ${useEmcc} -o:${outputFileName} ${inputFileName}`;
  console.info(`➤ ${command}`);
  await exec(command);
  return { outputFileName };
}
