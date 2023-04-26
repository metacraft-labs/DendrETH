import { exec as exec_, execSync } from 'node:child_process';
import { promisify } from 'node:util';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet, OfflineSigner } from '@cosmjs/proto-signing';
import { getRootDir, sleep } from '../ts-utils/common-utils';
import { cosmosUtils, cosmosWalletInfo } from './cosmos-utils';

const exec = promisify(exec_);

export async function setUpCosmosTestnet(
  rpcEndpoint: string,
  signal: AbortSignal,
) {
  let DendrETHWalletInfo = new cosmosWalletInfo(
    String(process.env['LOCAL_COSMOS_MNEMONIC']),
  );
  const rootDir = await getRootDir();
  const setupWasmdCommand = `bash "${rootDir}/contracts/cosmos/scripts/setup_wasmd.sh"`;
  console.info(`Preparing 'wasmd'. \n  ╰─➤ ${setupWasmdCommand}`);
  execSync(setupWasmdCommand);

  const startNodeCommand = `bash "${rootDir}/contracts/cosmos/scripts/start_node.sh"`;
  console.info(`Starting Cosmos testnet node. \n  ╰─➤ ${startNodeCommand}`);
  exec_(startNodeCommand, { signal });

  await sleep(15000); //  Make sure the node has started

  const addKeyCommand = `bash "${rootDir}/contracts/cosmos/scripts/add_account.sh"`;
  console.info(`Creating and funding account. \n  ╰─➤ ${addKeyCommand}`);
  execSync(addKeyCommand);

  await sleep(10000); //  Make sure the new account is funded

  const getFredAddressCommand = `wasmd keys show fred -a --keyring-backend test \
    --keyring-dir $HOME/.wasmd_keys`;
  console.info(`Get funded account data. \n  ╰─➤ ${getFredAddressCommand}`);
  const getAddress = exec(getFredAddressCommand);
  const fredAddress = (await getAddress).stdout;
  DendrETHWalletInfo.address = fredAddress.trimEnd();
  let wallet = await DirectSecp256k1HdWallet.fromMnemonic(
    DendrETHWalletInfo.mnemonic,
    {
      prefix: 'wasm',
    },
  );

  let client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
  );
  let cosmos = new cosmosUtils(client, DendrETHWalletInfo);
  return cosmos;
}
