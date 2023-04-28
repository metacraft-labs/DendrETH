import { exec as exec_, execSync } from 'node:child_process';
import { promisify } from 'node:util';

import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';

const exec = promisify(exec_);

export class cosmosWalletInfo {
  mnemonic: string;
  address: string;

  constructor(mnemonic: string = '', address: string = '') {
    this.mnemonic = mnemonic;
    this.address = address;
  }
}

// Class to use as result for the function below
export class cosmosUtils {
  client: SigningCosmWasmClient;
  walletInfo: cosmosWalletInfo;

  constructor(client: SigningCosmWasmClient, walletInfo: cosmosWalletInfo) {
    this.client = client;
    this.walletInfo = walletInfo;
  }
}

export async function initCosmosUtils(network: string) {
  switch (network) {
    case 'cudos': {
      const rpcEndpoint = String(process.env['CUDOS_RPC_ENDPOINT']);
      const DendrETHWalletInfo = {
        mnemonic: String(process.env['CUDOS_MNEMONIC']),
        address: String(process.env['CUDOS_PUBLIC_KEY']),
      };
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        DendrETHWalletInfo.mnemonic,
        {
          prefix: 'cudos',
        },
      );
      const client = await SigningCosmWasmClient.connectWithSigner(
        rpcEndpoint,
        wallet,
        {
          gasPrice: GasPrice.fromString('5000000000000acudos'),
        },
      );
      let cosmos = new cosmosUtils(client, DendrETHWalletInfo);
      return cosmos;
    }
    case 'local': {
      const rpcEndpoint = String(process.env['COSMOS_LOCAL_RPC_ENDPOINT']);
      const getFredAddressCommand = `wasmd keys show fred -a --keyring-backend test \
      --keyring-dir $HOME/.wasmd_keys`;
      console.info(`Get funded account data. \n  ╰─➤ ${getFredAddressCommand}`);
      const getAddress = exec(getFredAddressCommand);
      const fredAddress = (await getAddress).stdout;
      const DendrETHWalletInfo = {
        mnemonic: String(process.env['COSMOS_LOCAL_MNEMONIC']),
        address: fredAddress.trimEnd(),
      };
      let wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        DendrETHWalletInfo.mnemonic,
        {
          prefix: 'wasm',
        },
      );
      var client;
      try {
        client = await SigningCosmWasmClient.connectWithSigner(
          rpcEndpoint,
          wallet,
        );
      } catch (e) {
        console.log(e);
        console.log('\n ERROR:Testnet is not up!!! \n');
      }
      let cosmos = new cosmosUtils(client, DendrETHWalletInfo);
      return cosmos;
    }
    default: {
      console.error('Incorrect network parameter');
      return;
    }
  }
}
