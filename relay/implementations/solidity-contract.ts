import { Contract } from 'ethers';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { groth16 } from 'snarkjs';
import Web3 from 'web3';
import {
  TransactionSpeed,
  publishTransaction,
} from './publish_evm_transaction';

export class SolidityContract implements ISmartContract {
  private lightClientContract: Contract;
  private web3: Web3;
  private transactionSpeed: TransactionSpeed;

  constructor(
    lightClientContract: Contract,
    rpcEndpoint: string,
    transactionSpeed: TransactionSpeed = 'avg',
  ) {
    this.lightClientContract = lightClientContract;
    this.web3 = new Web3(rpcEndpoint);
    this.transactionSpeed = transactionSpeed;
  }

  optimisticHeaderRoot(): Promise<string> {
    return this.lightClientContract.optimisticHeaderRoot();
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
    const calldata = await groth16.exportSolidityCallData(
      {
        pi_a: update.a,
        pi_b: update.b,
        pi_c: update.c,
      },
      [],
    );

    const argv: string[] = calldata
      .replace(/["[\]\s]/g, '')
      .split(',')
      .map(x => BigInt(x).toString());

    const a = [argv[0], argv[1]];
    const b = [
      [argv[2], argv[3]],
      [argv[4], argv[5]],
    ];
    const c = [argv[6], argv[7]];

    await publishTransaction(
      this.lightClientContract,
      'light_client_update',
      {
        ...update,
        a,
        b,
        c,
      },
      this.web3,
      this.transactionSpeed,
    );
  }
}
