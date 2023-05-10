import { Contract, ethers } from 'ethers';
import { parseEther } from 'ethers/lib/utils';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';

export class SolidityContract implements ISmartContract {
  private lightClientContract: Contract;

  constructor(lightClientContract: Contract) {
    this.lightClientContract = lightClientContract;
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
    return this.lightClientContract.light_client_update(update);
  }
}
