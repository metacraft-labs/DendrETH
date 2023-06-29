import { exec as exec_, execSync } from 'node:child_process';
import { promisify } from 'node:util';

import {
  CosmosClientWithWallet,
  getCosmosContractArtifacts,
  getCosmosTxClient,
} from './cosmos-utils';

const exec = promisify(exec_);

export async function startCosmosNode(target: string) {
  const localNodeUrl = 'http://localhost:26657';
  const { contractDir } = await getCosmosContractArtifacts(target);
  const startNodeCommand = `bash ${contractDir}/../scripts/run_cosmos_node.sh start`;
  console.info(`Starting Cosmos node. \n  ╰─➤ ${startNodeCommand}`);
  execSync(startNodeCommand);

  return localNodeUrl;
}

export async function stopCosmosNode(target: string) {
  const { contractDir } = await getCosmosContractArtifacts(target);
  const stopNodeCommand = `bash ${contractDir}/../scripts/run_cosmos_node.sh stop`;
  console.info(`Stopping Cosmos node. \n  ╰─➤ ${stopNodeCommand}`);
  execSync(stopNodeCommand);
}

export async function setUpCosmosTestnet(
  mnemonic: string,
  target: string,
): Promise<CosmosClientWithWallet> {
  const rpcEndpoint = await startCosmosNode(target);
  return getCosmosTxClient(mnemonic, 'wasm', rpcEndpoint);
}
