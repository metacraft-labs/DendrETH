import yargs from 'yargs/yargs';
import { getCosmosTxClient } from '../../../libs/typescript/cosmos-utils/cosmos-utils';
import {
  startCosmosNode,
  stopCosmosNode,
} from '../../../libs/typescript/cosmos-utils/testnet-setup';
import {
  instantiateVerifierContract,
  uploadVerifierContract,
} from '../verifier/lib/typescript/verifier-upload-instantiate';

const argv = yargs(process.argv.slice(2))
  .options({
    run: { type: 'boolean', default: false, demandOption: true },
    network: { type: 'string', demandOption: true },
    mnemonic: { type: 'string', demandOption: true },
    rpcUrl: { type: 'string', demandOption: true },
    initHeaderRoot: { type: 'string' },
    startTestnet: { type: 'boolean', default: false },
    terminateTestnet: { type: 'boolean', default: false },
  })
  .parseSync();

async function uploadAndInstantiateMain() {
  const network = argv.network;
  const mnemonic = argv.mnemonic;
  let rpcUrl = argv.rpcUrl;

  if (network === 'wasm' && argv.startTestnet) {
    // This way we are able to run the script without starting the testnet separately
    rpcUrl = await startCosmosNode();
  }
  const cosmos = await getCosmosTxClient(mnemonic, network, rpcUrl);

  const uploadReceipt = await uploadVerifierContract(network, cosmos);
  if (!uploadReceipt) {
    console.error('Upload failed');
    return;
  }
  // as default we use root of this header
  // http://unstable.prater.beacon-api.nimbus.team/eth/v1/beacon/headers/5200024
  const defaultInitHeaderRoot =
    '0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316';
  const initHeaderRoot = argv.initHeaderRoot || defaultInitHeaderRoot;
  await instantiateVerifierContract(uploadReceipt, initHeaderRoot, cosmos);

  if (network === 'local' && argv.terminateTestnet === true) {
    await stopCosmosNode();
  }
}
if (argv.run || argv._[0] == 'run') {
  uploadAndInstantiateMain();
}
