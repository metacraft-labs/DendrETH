import { BigNumber, Contract } from 'ethers';
import Web3 from 'web3';
import { FeeHistoryResult } from 'web3-eth';
import { groth16 } from 'snarkjs';

type Block = {
  number: number | string;
  baseFeePerGas: number;
  gasUsedRatio: number;
  priorityFeePerGas: number[];
};

export type TransactionSpeed = 'slow' | 'avg' | 'fast';

const historicalBlocks = 20;

export async function publishTransaction(
  contract: Contract,
  method: string,
  params: any,
  web3: Web3,
  transactionSpeed: TransactionSpeed,
  spread?: boolean,
) {
  const transactionCount = await contract.signer.getTransactionCount();

  let transactionPromise;
  let retries = 1;

  while (true) {
    try {
      const { priorityFeePerGas, baseFeePerGas } = await getGasPrice(
        web3,
        transactionSpeed,
      );

      const priorityFeePerGasNumber =
        BigNumber.from(priorityFeePerGas).mul(retries);

      const transactionData = {
        nonce: transactionCount,
        // Multiply by 2 in case of base fee spike as the unused gas will be returned
        maxFeePerGas: priorityFeePerGasNumber.add(baseFeePerGas).mul(2),
        maxPriorityFeePerGas: priorityFeePerGasNumber,
      };

      console.log(transactionData);

      let estimateGas;
      let transaction;
      if (spread) {
        estimateGas = await contract.estimateGas[method](...params);

        transaction = await contract[method](...params, {
          nonce: transactionData.nonce,
          maxFeePerGas: transactionData.maxFeePerGas,
          maxPriorityFeePerGas: transactionData.maxPriorityFeePerGas,
          gasLimit: estimateGas,
        });
      } else {
        estimateGas = await contract.estimateGas[method](params);

        transaction = await contract[method](params, {
          nonce: transactionData.nonce,
          maxFeePerGas: transactionData.maxFeePerGas,
          maxPriorityFeePerGas: transactionData.maxPriorityFeePerGas,
          gasLimit: estimateGas,
        });
      }

      console.log(transaction);

      transactionPromise = transaction.wait();
    } catch (e) {
      if (e instanceof Error && e.message.includes('eth_feeHistory')) {
        const transaction = await contract[method](params, {
          nonce: transactionCount,
          gasPrice: (await contract.provider.getGasPrice())
            .mul(10 + retries)
            .div(10),
        });

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

export async function getSolidityProof(update: {
  a: string[];
  b: string[][];
  c: string[];
}): Promise<{
  a: string[];
  b: string[][];
  c: string[];
}> {
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
  return { a, b, c };
}

async function getGasPrice(web3: Web3, transactionSpeed: TransactionSpeed) {
  const formattedBlocks = formatFeeHistory(
    await web3.eth.getFeeHistory(historicalBlocks, 'pending', [1, 50, 99]),
    false,
  );

  const slow = avg(formattedBlocks.map(b => b.priorityFeePerGas[0]));

  const average = avg(formattedBlocks.map(b => b.priorityFeePerGas[1]));

  const fast = avg(formattedBlocks.map(b => b.priorityFeePerGas[2]));

  const getPriorityFeePerGas = () => {
    switch (transactionSpeed) {
      case 'slow':
        return slow;
      case 'avg':
        return average;
      case 'fast':
        return fast;
    }
  };

  const baseFeePerGas = (await web3.eth.getBlock('pending')).baseFeePerGas!;
  return { priorityFeePerGas: getPriorityFeePerGas(), baseFeePerGas };
}

function formatFeeHistory(
  result: FeeHistoryResult,
  includePending: boolean,
): Block[] {
  let blockNum = Number(result.oldestBlock);

  const blocks: Block[] = [];

  for (let i = 0; i < historicalBlocks; i++) {
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
      baseFeePerGas: Number(result.baseFeePerGas[historicalBlocks]),
      gasUsedRatio: NaN,
      priorityFeePerGas: [],
    });
  }

  return blocks;
}

function avg(arr) {
  const sum = arr.reduce((a, v) => a + v);
  return Math.round(sum / arr.length);
}
