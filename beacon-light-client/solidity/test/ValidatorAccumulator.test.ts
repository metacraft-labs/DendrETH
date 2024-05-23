import { ethers, network } from 'hardhat';
import { Contract } from 'ethers';
import { sha256 } from 'ethers';
import { hashTreeRoot } from '@dendreth/utils/ts-utils/ssz-utils';
import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { expect } from 'chai';
import depositContractAbi from './abis/deposit.json';
import depositItems from './utils/depositData.json';

export function calculateValidatorsAccumulator(
  validatorsPubkeys: Uint8Array[],
  eth1DepositIndexes: Uint8Array[],
  depositMessageRoots: Uint8Array[],
  signatures: Uint8Array[],
): string {
  const leaves = validatorsPubkeys.map((pubkey, i) => {
    const validatorPubkey = bytesToHex(pubkey);
    const eth1DepositIndex = bytesToHex(eth1DepositIndexes[i]);
    const depositMessageRoot = bytesToHex(depositMessageRoots[i]);
    const signature = bytesToHex(signatures[i]);

    return sha256(
      '0x' +
        formatHex(validatorPubkey) +
        formatHex(eth1DepositIndex) +
        formatHex(depositMessageRoot) +
        formatHex(signature),
    );
  });

  return hashTreeRoot(leaves, 32);
}

describe('ValidatorsAccumulator tests', async function () {
  let validatorAccumulator: Contract;
  let depositContract: Contract;
  let pubkeys: Uint8Array[] = [];
  let eth1DepositIndexes: Uint8Array[] = [];
  let depositMessageRoots: Uint8Array[] = [];
  let signatures: Uint8Array[] = [];
  let accumulators: string[] = [];

  beforeEach(async function () {
    pubkeys = [];
    eth1DepositIndexes = [];
    depositMessageRoots = [];
    signatures = [];
    accumulators = [];

    depositContract = await ethers.getContractAt(
      depositContractAbi,
      '0x00000000219ab540356cBB839Cbe05303d7705Fa',
    );

    const signerAddress = (await ethers.provider.getSigner(0)).address;

    const contractFactory = await ethers.getContractFactory(
      'ValidatorsAccumulator',
    );
    validatorAccumulator = await contractFactory.deploy(
      '0x00000000219ab540356cBB839Cbe05303d7705Fa',
      signerAddress,
    );

    await network.provider.send('hardhat_setBalance', [
      signerAddress,
      ethers.toBeHex(7119834272032510088813n),
    ]);
  });

  async function deposit(depositItem: any) {
    const { ssz } = await import('@lodestar/types');

    await (
      await validatorAccumulator.deposit(
        depositItem.pubkey,
        depositItem.withdrawalCredentials,
        depositItem.signature,
        depositItem.depositDataRoot,
        { value: ethers.parseEther('32').toString() },
      )
    ).wait();

    const pubkey = hexToBytes(depositItem.pubkey);
    const eth1DepositIndex = await depositContract.get_deposit_count();

    let deposit_message = {
      pubkey: hexToBytes(depositItem.pubkey),
      withdrawalCredentials: hexToBytes(depositItem.withdrawalCredentials),
      amount: 32000000000,
    };
    const depositMessageRoot =
      ssz.phase0.DepositMessage.hashTreeRoot(deposit_message);
    const signature = hexToBytes(depositItem.signature);

    accumulators.push(await validatorAccumulator.getValidatorsAccumulator());
    pubkeys.push(pubkey);
    eth1DepositIndexes.push(hexToBytes(eth1DepositIndex));
    depositMessageRoots.push(depositMessageRoot);
    signatures.push(signature);
  }

  it('Should deposit', async function () {
    for (const depositItem of depositItems) {
      await deposit(depositItem);
      expect(await validatorAccumulator.getValidatorsAccumulator()).to.equal(
        calculateValidatorsAccumulator(
          pubkeys,
          eth1DepositIndexes,
          depositMessageRoots,
          signatures,
        ),
      );
    }
  });

  it('Should find correct accumulator in consecutive blocks and prune all before it', async function () {
    const startBlock =
      Number(await network.provider.send('eth_blockNumber')) + 1;
    for (const depositItem of depositItems) {
      await deposit(depositItem);
    }

    const index = Math.floor(depositItems.length / 2);

    const accumulator = await validatorAccumulator.findAndPruneBlock.staticCall(
      startBlock + index,
    );

    expect(accumulator).to.equal(accumulators[index]);
  });

  it('Should find correct accumulator in non-consecutive blocks and prune all before it', async function () {
    await deposit(depositItems[0]);
    await network.provider.send('hardhat_mine', [ethers.toBeHex(2)]);
    const startBlock0 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[1]);
    await network.provider.send('hardhat_mine', [ethers.toBeHex(5)]);
    const startBlock1 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[2]);
    const startBlock2 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[3]);
    await network.provider.send('hardhat_mine', [ethers.toBeHex(10)]);
    const startBlock3 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[4]);
    const startBlock4 = Number(await network.provider.send('eth_blockNumber'));

    const resAccumulators = [
      await validatorAccumulator.findAndPruneBlock.staticCall(startBlock0),
      await validatorAccumulator.findAndPruneBlock.staticCall(startBlock1),
      await validatorAccumulator.findAndPruneBlock.staticCall(startBlock2),
      await validatorAccumulator.findAndPruneBlock.staticCall(startBlock3),
      await validatorAccumulator.findAndPruneBlock.staticCall(startBlock4),
    ];

    for (let i = 0; i < resAccumulators.length; i++) {
      expect(resAccumulators[i]).to.equal(accumulators[i]);
    }
  });

  it('Should not find one accumulator twice', async function () {
    const startBlock =
      Number(await network.provider.send('eth_blockNumber')) + 1;
    for (const depositItem of depositItems) {
      await deposit(depositItem);
    }

    const index = Math.floor(depositItems.length / 2);

    const accumulator = await validatorAccumulator.findAndPruneBlock.staticCall(
      startBlock + index,
    );
    await validatorAccumulator.findAndPruneBlock(startBlock + index);

    expect(accumulator).to.equal(accumulators[index]);

    const accumulator1 =
      await validatorAccumulator.findAndPruneBlock.staticCall(
        startBlock + index,
      );

    expect(accumulator1).to.equal(ethers.toBeHex('0').padEnd(66, '0'));
  });
});
