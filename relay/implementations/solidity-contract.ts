import { Contract, ethers } from 'ethers';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';

export class SolidityContract implements ISmartContract {
  private lightClientContract: Contract;

  constructor(lightClientContract: Contract) {
    this.lightClientContract = lightClientContract;
  }

  optimisticHeaderRoot(): Promise<string> {
    return this.lightClientContract.optimistic_header_root();
  }
  postUpdateOnChain(update: {
    attested_header_root: string;
    finalized_header_root: string;
    finalized_execution_state_root: string;
    a: string[];
    b: string[][];
    c: string[];
  }): Promise<any> {
    return this.lightClientContract.light_client_update(update);
  }
}
