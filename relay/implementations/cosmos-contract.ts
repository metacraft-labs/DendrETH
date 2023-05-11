import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { promisify } from 'node:util';
import { exec as exec_, execSync, spawn } from 'node:child_process';
import { calculateFee, GasPrice } from '@cosmjs/stargate';

const exec = promisify(exec_);

export class CosmosContract implements ISmartContract {
  private contractAddress: string;
  private callerAddress: string;
  private rpcEndpoint: string;
  private client: SigningCosmWasmClient | undefined;
  private network;

  constructor(
    contractAddress: string,
    callerAddress: string,
    rpcEndpoint: string,
    network: string,
  ) {
    this.contractAddress = contractAddress;
    this.callerAddress = callerAddress;
    this.rpcEndpoint = rpcEndpoint;
    this.network = network;
  }

  async getClient() {
    if (this.client) {
      return this.client;
    }
    switch (this.network) {
      case 'cudos': {
        this.client = await CreateClientCudos(this.rpcEndpoint);
        break;
      }
      case 'malaga': {
        this.client = await CreateClientMalaga(this.rpcEndpoint);
        break;
      }
      case 'local': {
        this.client = await CreateClientLocal(this.rpcEndpoint);
        break;
      }
    }
    return this.client;
  }

  async optimisticHeaderRoot(): Promise<string> {
    let lastHeader;
    const client = await this.getClient();
    if (client) {
      lastHeader = await client.queryContractSmart(this.contractAddress, {
        last_header_hash: {},
      });
    } else {
      console.error('Failed to create client');
    }
    return lastHeader;
  }

  async postUpdateOnChain(update: {
    attestedHeaderRoot: string;
    attestedHeaderSlot: number;
    finalizedHeaderRoot: string;
    finalizedExecutionStateRoot: string;
    a: string[];
    b: string[][];
    c: string[];
  }): Promise<any> {
    const rootDir = (
      await exec('git rev-parse --show-toplevel')
    ).stdout.replace(/\s/g, '');
    const parseDataTool =
      rootDir + `/contracts/cosmos/verifier/nimcache/verifier_parse_data`;

    const flattedB = update.b.flat();
    const parseUpdateDataCommand = `${parseDataTool} updateDataForCosmosContractClass \
  --attested_header_root=${update.attestedHeaderRoot} --finalized_header_root=${update.finalizedHeaderRoot} --finalized_execution_state_root= ${update.finalizedExecutionStateRoot} \
  --a=${update.a[0]} --a=${update.a[1]} --a=${update.a[2]} \
  --b=${flattedB[0]} --b=${flattedB[1]} --b=${flattedB[2]} --b=${flattedB[3]} --b=${flattedB[4]} --b=${flattedB[5]} \
  --c=${update.c[0]} --c=${update.c[1]} --c=${update.c[2]} `;
    const updateDataExec = exec(parseUpdateDataCommand);
    const updateData = (await updateDataExec).stdout.replace(/\s/g, '');

    var executeFee;
    switch (this.network) {
      case 'cudos': {
        executeFee = 'auto';
        break;
      }
      case 'malaga': {
        executeFee = 'auto';
        break;
      }
      case 'local': {
        const gasPrice = GasPrice.fromString('0.0000025ustake');
        executeFee = calculateFee(2_000_000, gasPrice);
        break;
      }
    }
    let result;
    const client = await this.getClient();
    if (client) {
      result = await client.execute(
        this.callerAddress,
        this.contractAddress,
        JSON.parse(updateData),
        executeFee,
        'Updating the Verifier',
      );
    } else {
      console.error('Failed to create client');
    }
    return result;
  }
}
async function CreateClientCudos(rpcEndpoint) {
  const mnemonic = String(process.env['CUDOS_MNEMONIC']);
  console.log(mnemonic);
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: 'cudos',
  });
  var client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
    {
      gasPrice: GasPrice.fromString('10000000000000acudos'),
    },
  );
  return client;
}
async function CreateClientMalaga(rpcEndpoint) {
  const mnemonic = String(process.env['MALAGA_MNEMONIC']);
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: 'wasm',
  });
  var client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
    {
      gasPrice: GasPrice.fromString('0.5umlg'),
    },
  );
  return client;
}

async function CreateClientLocal(rpcEndpoint) {
  const mnemonic =
    'economy stock theory fatal elder harbor betray wasp final emotion task crumble siren bottom lizard educate guess current outdoor pair theory focus wife stone';

  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: 'wasm',
  });
  var client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
  );
  return client;
}
