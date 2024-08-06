import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { rootDir } from '../ts-utils/common-utils';

export async function getCosmosContractArtifacts(contract: string) {
  var contractDir;
  if (contract == 'light-client') {
    contractDir = `${rootDir}/contracts/cosmos/${contract}`;
  } else {
    contractDir = `${rootDir}/contracts/cosmos/verifier/${contract}`;
  }
  const wasmContractPath = `${contractDir}/artifacts/verifier.wasm`;

  return { rootDir, contractDir, wasmContractPath };
}
export interface CosmosWalletInfo {
  mnemonic: string;
  address: string;
}

// Class to use as result for the function below
export interface CosmosClientWithWallet {
  client: SigningCosmWasmClient;
  walletInfo: CosmosWalletInfo;
}

export async function getCosmosTxClient(
  mnemonic: string,
  network: string,
  rpcUrl: string,
): Promise<CosmosClientWithWallet> {
  const prefix = network === 'cudos' ? 'cudos' : 'wasm';
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: prefix,
  });
  const accounts = await wallet.getAccounts();
  const address = accounts[0].address;

  let client;
  switch (network) {
    default: {
      throw new Error('Incorrect network parameter');
    }
    case 'cudos': {
      client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, wallet, {
        gasPrice: GasPrice.fromString('5000000000000acudos'),
      });
      break;
    }
    case 'malaga': {
      client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, wallet, {
        gasPrice: GasPrice.fromString('0.5umlg'),
      });
      break;
    }
    case 'wasm': {
      try {
        client = await SigningCosmWasmClient.connectWithSigner(rpcUrl, wallet);
      } catch (e) {
        throw new Error('Could not connect to testnet' + e);
      }
      break;
    }
  }

  return {
    client,
    walletInfo: {
      mnemonic,
      address,
    },
  };
}

export function getDataFromPrintHeaderResult(result: string) {
  const noSpaces = result.replace(/\s/g, '');
  const index = noSpaces.indexOf('>>');
  return noSpaces.slice(index + 2);
}
