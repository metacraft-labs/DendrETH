import { ethers, network } from 'hardhat';
import { Contract } from 'ethers';
import { sha256 } from 'ethers/lib/utils';
import { hashTreeRoot } from '@dendreth/utils/ts-utils/ssz-utils';
import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { expect } from 'chai';
import depositItems from './utils/depositData.json';

describe('ValidatorsAccumulator tests', async function () {
  let validatorAccumulator: Contract;
  let pubkeys: Uint8Array[] = [];
  let accumulators: string[] = [];

  beforeEach(async function () {
    pubkeys = [];
    accumulators = [];

    const signerAddress = (await ethers.getSigners())[0].address;

    const contractFactory = await ethers.getContractFactory(
      'ValidatorsAccumulator',
    );
    validatorAccumulator = await contractFactory.deploy(
      '0x00000000219ab540356cBB839Cbe05303d7705Fa',
    );

    await network.provider.send('hardhat_setBalance', [
      signerAddress,
      ethers.utils.hexValue(7119834272032510088813n),
    ]);
  });

  async function deposit(depositItem: any) {
    await (
      await validatorAccumulator.deposit(
        depositItem.pubkey,
        depositItem.withdrawalCredentials,
        depositItem.signature,
        depositItem.depositDataRoot,
        { value: ethers.utils.parseEther('32').toString() },
      )
    ).wait();

    const pubkey = hexToBytes(depositItem.pubkey);

    accumulators.push(await validatorAccumulator.getValidatorsAccumulator());
    pubkeys.push(pubkey);
  }

  function calculateValidatorsAccumulator(
    validatorsPubkeys: Uint8Array[],
  ): string {
    const leaves = validatorsPubkeys.map((pubkey, i) => {
      const validatorPubkey = bytesToHex(pubkey);

      return sha256('0x' + formatHex(validatorPubkey));
    });

    return hashTreeRoot(leaves, 32);
  }

  it('Should deposit', async function () {
    for (const depositItem of depositItems) {
      await deposit(depositItem);
      expect(await validatorAccumulator.getValidatorsAccumulator()).to.equal(
        calculateValidatorsAccumulator(pubkeys),
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

    const [count, accumulator] =
      await validatorAccumulator.findAccumulatorByBlock(startBlock + index);

    expect(accumulator).to.equal(accumulators[index]);
    expect(count.toNumber()).to.equal(index + 1);
  });

  it('Should find correct accumulator in non-consecutive blocks and prune all before it', async function () {
    await deposit(depositItems[0]);
    await network.provider.send('hardhat_mine', [ethers.utils.hexValue(2)]);
    const startBlock0 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[1]);
    await network.provider.send('hardhat_mine', [ethers.utils.hexValue(5)]);
    const startBlock1 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[2]);
    const startBlock2 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[3]);
    await network.provider.send('hardhat_mine', [ethers.utils.hexValue(10)]);
    const startBlock3 = Number(await network.provider.send('eth_blockNumber'));
    await deposit(depositItems[4]);
    const startBlock4 = Number(await network.provider.send('eth_blockNumber'));

    const res = await Promise.all([
      validatorAccumulator.findAccumulatorByBlock(startBlock0),
      validatorAccumulator.findAccumulatorByBlock(startBlock1),
      validatorAccumulator.findAccumulatorByBlock(startBlock2),
      validatorAccumulator.findAccumulatorByBlock(startBlock3),
      validatorAccumulator.findAccumulatorByBlock(startBlock4),
    ]);

    for (let i = 0; i < res.length; i++) {
      expect(res[i][0].toNumber()).to.equal(i + 1);
      expect(res[i][1]).to.equal(accumulators[i]);
    }
  });

  it('Should return zero hash if block is before first deposit', async function () {
    await deposit(depositItems[0]);
    const startBlock = Number(await network.provider.send('eth_blockNumber'));

    const [count, accumulator] =
      await validatorAccumulator.findAccumulatorByBlock(startBlock - 1);

    expect(accumulator).to.equal(
      '0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7',
    );
    expect(count.toNumber()).to.equal(0);
  });

  it('Should return zero hash if no validators have deposited', async function () {
    const startBlock = Number(await network.provider.send('eth_blockNumber'));

    const [count, accumulator] =
      await validatorAccumulator.findAccumulatorByBlock(startBlock);

    expect(accumulator).to.equal(
      '0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7',
    );
    expect(count.toNumber()).to.equal(0);
  });
});
