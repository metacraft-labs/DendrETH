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
    domain: { type: 'string' },
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
    '0x4ce76b7478cb0eee4a32c7f25bb561ca1d0f444d1716c8f6f260900ef45f37d2';
  const defaultDomain =
    '0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695';
  const initHeaderRoot = argv.initHeaderRoot || defaultInitHeaderRoot;
  const domain = argv.domain || defaultDomain;
  await instantiateVerifierContract(
    uploadReceipt,
    initHeaderRoot,
    domain,
    cosmos,
  );

  if (network === 'wasm' && argv.terminateTestnet === true) {
    await stopCosmosNode();
  }
}
if (argv.run || argv._[0] == 'run') {
  uploadAndInstantiateMain();
}
