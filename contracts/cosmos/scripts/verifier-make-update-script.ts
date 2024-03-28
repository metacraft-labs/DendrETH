import yargs from 'yargs/yargs';
import { getCosmosTxClient } from '@dendreth/utils/cosmos-utils/cosmos-utils';
import { updateVerifierContract } from '../verifier/typescript/verifier-make-update';

const argv = yargs(process.argv.slice(2))
  .options({
    run: { type: 'boolean', default: false, demandOption: true },
    target: {
      type: 'string',
      demandOption: true,
      choices: ['verifier-bncurve', 'verifier-constantine'],
    },
    network: {
      type: 'string',
      demandOption: true,
      choices: ['wasm', 'cudos', 'malaga'],
    },
    mnemonic: { type: 'string', demandOption: true },
    rpcUrl: { type: 'string', demandOption: true },
    contractAddress: { type: 'string', demandOption: true },
    updateNum: { type: 'string' },
  })
  .parseSync();

async function uploadMain() {
  // as default we will use the first update file we have
  let defaultUpdateFile = '5200024_5200056.json';

  const target = argv.target;
  const network = argv.network;
  const mnemonic = argv.mnemonic;
  const rpcUrl = argv.rpcUrl;
  const contractAddress = argv.contractAddress;
  const updateFile = argv.updateNum || defaultUpdateFile;
  const cosmos = await getCosmosTxClient(mnemonic, network, rpcUrl);

  if (!cosmos) {
    console.error('Cosmos client and wallet failed to initialize');
    return;
  }

  await updateVerifierContract(
    network,
    cosmos,
    contractAddress,
    updateFile,
    target,
  );
}
if (argv.run || argv._[0] == 'run') {
  uploadMain();
}
