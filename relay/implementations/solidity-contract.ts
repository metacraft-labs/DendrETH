import { Contract, ethers } from 'ethers';
import { parseEther } from 'ethers/lib/utils';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { groth16 } from 'snarkjs';

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

    const transaction = await this.lightClientContract.light_client_update({
      ...update,
      a,
      b,
      c,
    });

    console.log(transaction);

    await transaction.wait();
  }
}
