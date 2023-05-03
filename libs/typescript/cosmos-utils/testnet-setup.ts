import { exec as exec_, execSync } from 'node:child_process';
import { promisify } from 'node:util';

import {
  CosmosClientWithWallet,
  getCosmosContractArtifacts,
  getCosmosTxClient,
} from './cosmos-utils';

const exec = promisify(exec_);

export async function startCosmosNode() {
  const localNodeUrl = 'http://localhost:26657';
  const { contractDir } = await getCosmosContractArtifacts('verifier');
  const startNodeCommand = `bash ${contractDir}/../scripts/run_cosmos_node.sh start`;
  console.info(`Starting Cosmos node. \n  ╰─➤ ${startNodeCommand}`);
  execSync(startNodeCommand);

  return localNodeUrl;
}

export async function stopCosmosNode() {
  const { contractDir } = await getCosmosContractArtifacts('verifier');
  const stopNodeCommand = `bash ${contractDir}/../scripts/run_cosmos_node.sh stop`;
  console.info(`Stopping Cosmos node. \n  ╰─➤ ${stopNodeCommand}`);
  execSync(stopNodeCommand);
}

export async function setUpCosmosTestnet(
  mnemonic: string,
): Promise<CosmosClientWithWallet> {
  const rpcEndpoint = await startCosmosNode();
  return getCosmosTxClient(mnemonic, 'wasm', rpcEndpoint);
}
