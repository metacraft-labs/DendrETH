import { Contract } from 'ethers';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import Web3 from 'web3';
import {
  TransactionSpeed,
  getSolidityProof,
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

  optimisticHeaderSlot(): Promise<number> {
    return this.lightClientContract.optimisticHeaderSlot();
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
    const { a, b, c } = await getSolidityProof(update);

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
