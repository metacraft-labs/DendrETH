import { BigNumber, Contract, ethers } from 'ethers';
import { parseEther } from 'ethers/lib/utils';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { groth16 } from 'snarkjs';
import Web3 from 'web3';
import { FeeHistoryResult } from 'web3-eth';

type Block = {
  number: number | string;
  baseFeePerGas: number;
  gasUsedRatio: number;
  priorityFeePerGas: number[];
};

type TransactionSpeed = 'slow' | 'avg' | 'fast';

export class SolidityContract implements ISmartContract {
  private static historicalBlocks = 20;

  private lightClientContract: Contract;
  private web3: Web3;
  private transactionSpeed: TransactionSpeed;

  constructor(
    lightClientContract: Contract,
    rpcEndpoint: string,
    transactionSpeed: TransactionSpeed = 'fast',
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

    const transactionCount =
      await this.lightClientContract.signer.getTransactionCount();

    let transactionPromise;
    let retries = 1;

    while (true) {
      try {
        const { priorityFeePerGas, baseFeePerGas } = await this.getGasPrice();

        const priorityFeePerGasNumber =
          BigNumber.from(priorityFeePerGas).mul(retries);

        const transactionData = {
          nonce: transactionCount,
          // Multiply by 2 in case of base fee spike as the unused gas will be returned
          maxFeePerGas: priorityFeePerGasNumber.add(baseFeePerGas).mul(2),
          maxPriorityFeePerGas: priorityFeePerGasNumber,
        };

        console.log(transactionData);

        const estimateGas =
          await this.lightClientContract.estimateGas.light_client_update({
            ...update,
            a,
            b,
            c,
          });

        const transaction = await this.lightClientContract.light_client_update(
          {
            ...update,
            a,
            b,
            c,
          },
          {
            nonce: transactionData.nonce,
            maxFeePerGas: transactionData.maxFeePerGas,
            maxPriorityFeePerGas: transactionData.maxPriorityFeePerGas,
            gasLimit: estimateGas,
          },
        );

        console.log(transaction);

        transactionPromise = transaction.wait();
      } catch (e) {
        if (e instanceof Error && e.message.includes('eth_feeHistory')) {
          const transaction =
            await this.lightClientContract.light_client_update(
              {
                ...update,
                a,
                b,
                c,
              },
              {
                nonce: transactionCount,
                gasPrice: (
                  await this.lightClientContract.provider.getGasPrice()
                )
                  .mul(10 + retries)
                  .div(10),
              },
            );

          console.log(transaction);

          transactionPromise = transaction.wait();
        } else {
          throw e;
        }
      }

      const r = await Promise.race([
        new Promise(r => setTimeout(r, 120000, 'unresolved')),
        transactionPromise,
      ]);

      if (r === 'unresolved') {
        console.log(
          'Transaction failed to be included in a block for 2 minutes retry with bumped fee',
        );
        retries++;
        continue;
      }

      break;
    }
  }

  private async getGasPrice() {
    const formatedBlocks = SolidityContract.formatFeeHistory(
      await this.web3.eth.getFeeHistory(
        SolidityContract.historicalBlocks,
        'pending',
        [1, 50, 99],
      ),
      false,
    );

    const slow = SolidityContract.avg(
      formatedBlocks.map(b => b.priorityFeePerGas[0]),
    );

    const average = SolidityContract.avg(
      formatedBlocks.map(b => b.priorityFeePerGas[1]),
    );

    const fast = SolidityContract.avg(
      formatedBlocks.map(b => b.priorityFeePerGas[2]),
    );

    const getPriorityFeePerGas = () => {
      switch (this.transactionSpeed) {
        case 'slow':
          return slow;
        case 'avg':
          return average;
        case 'fast':
          return fast;
      }
    };

    const baseFeePerGas = (await this.web3.eth.getBlock('pending'))
      .baseFeePerGas!;
    return { priorityFeePerGas: getPriorityFeePerGas(), baseFeePerGas };
  }

  private static formatFeeHistory(
    result: FeeHistoryResult,
    includePending: boolean,
  ): Block[] {
    let blockNum = Number(result.oldestBlock);

    const blocks: Block[] = [];

    for (let i = 0; i < SolidityContract.historicalBlocks; i++) {
      blocks.push({
        number: blockNum + i,
        baseFeePerGas: Number(result.baseFeePerGas[i]),
        gasUsedRatio: Number(result.gasUsedRatio[i]),
        priorityFeePerGas: result.reward[i].map(x => Number(x)),
      });
    }

    if (includePending) {
      blocks.push({
        number: 'pending',
        baseFeePerGas: Number(
          result.baseFeePerGas[SolidityContract.historicalBlocks],
        ),
        gasUsedRatio: NaN,
        priorityFeePerGas: [],
      });
    }

    return blocks;
  }

  private static avg(arr) {
    const sum = arr.reduce((a, v) => a + v);
    return Math.round(sum / arr.length);
  }
}
