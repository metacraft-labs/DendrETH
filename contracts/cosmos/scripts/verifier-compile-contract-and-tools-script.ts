import yargs from 'yargs/yargs';
import {
  compileVerifierContract,
  compileVerifierNimFileToWasm,
  compileVerifierParseDataTool,
} from '../verifier/lib/typescript/verifier-compile-contract-and-tools';

const argv = yargs(process.argv.slice(2))
  .options({
    run: { type: 'boolean', default: false, demandOption: true },
  })
  .parseSync();

async function compileContractMain() {
  await compileVerifierNimFileToWasm();
  await compileVerifierParseDataTool();
  await compileVerifierContract();
}

if (argv.run || argv._[0] == 'run') {
  compileContractMain();
}
