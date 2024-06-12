import { providers } from 'ethers';

export async function queryContractDeploymentBlockNumber(
  provider: providers.JsonRpcProvider,
  contractAddress: string,
): Promise<number | null> {
  const headBlock = await provider.getBlockNumber();

  let left = 0;
  let right = headBlock;

  let deploymentBlockNumber: number | null = null;

  while (left <= right) {
    const middle = left + Math.floor((right - left) / 2);
    const code = await provider.getCode(contractAddress, middle);

    if (code !== '0x') {
      deploymentBlockNumber = middle;
      right = middle - 1;
    } else {
      left = middle + 1;
    }
  }

  return deploymentBlockNumber;
}
