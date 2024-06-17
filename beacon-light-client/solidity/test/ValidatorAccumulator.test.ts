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

const zeroHash =
  '0x985e929f70af28d0bdd1a90a808f977f597c7c778c489e98d3bd8910d31ac0f7';

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

  async function depositTransaction(depositItem: any) {
    return validatorAccumulator.deposit(
      depositItem.pubkey,
      depositItem.withdrawalCredentials,
      depositItem.signature,
      depositItem.depositDataRoot,
      { value: ethers.utils.parseEther('32').toString() },
    );
  }

  async function deposit(depositItem: any) {
    const tx = await depositTransaction(depositItem);
    await tx.wait();

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

  it('Should find correct accumulator in consecutive blocks', async function () {
    const startBlock =
      Number(await network.provider.send('eth_blockNumber')) + 1;
    for (const depositItem of depositItems) {
      await deposit(depositItem);
    }

    const index = Math.floor(depositItems.length / 2);

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock + index,
    );

    expect(accumulator).to.equal(accumulators[index]);
  });

  it('Should find correct accumulator in non-consecutive blocks', async function () {
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
      expect(res[i]).to.equal(accumulators[i]);
    }
  });

  it('Should return zero hash if block is before first deposit', async function () {
    await deposit(depositItems[0]);
    const startBlock = Number(await network.provider.send('eth_blockNumber'));

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock - 1,
    );

    expect(accumulator).to.equal(zeroHash);
  });

  it('Should return zero hash if no validators have deposited', async function () {
    const startBlock = Number(await network.provider.send('eth_blockNumber'));

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock,
    );

    expect(accumulator).to.equal(zeroHash);
  });

  it('Should return latest value for block number in the middle', async function () {
    await deposit(depositItems[4]);

    const startBlock = Number(await network.provider.send('eth_blockNumber'));
    await network.provider.send('evm_setAutomine', [false]);

    await depositTransaction(depositItems[0]);
    await depositTransaction(depositItems[1]);
    await depositTransaction(depositItems[2]);

    await network.provider.send('evm_mine');
    await network.provider.send('evm_setAutomine', [true]);

    await deposit(depositItems[6]);

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock + 1,
    );

    expect(accumulator).to.equal(
      '0xd03564681d03f73e2fa4e5092dcb54fe0d0adb996613fff319183da3b01d3b2d',
    );
  });

  it('Should return latest value for block number in the beginning', async function () {
    const startBlock = Number(await network.provider.send('eth_blockNumber'));
    await network.provider.send('evm_setAutomine', [false]);

    await depositTransaction(depositItems[0]);
    await depositTransaction(depositItems[1]);
    await depositTransaction(depositItems[2]);

    await network.provider.send('evm_mine');
    await network.provider.send('evm_setAutomine', [true]);

    for (const depositItem of depositItems.slice(7)) {
      await deposit(depositItem);
    }

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock + 1,
    );

    expect(accumulator).to.equal(
      '0x1e1f85c01394547df1c4fc661839a8fbaf63a9bb0c1d819785f549d9f81eeca0',
    );
  });

  it('Should return latest value for block number at the end', async function () {
    for (const depositItem of depositItems.slice(7)) {
      await deposit(depositItem);
    }

    const startBlock = Number(await network.provider.send('eth_blockNumber'));
    await network.provider.send('evm_setAutomine', [false]);

    await depositTransaction(depositItems[0]);
    await depositTransaction(depositItems[2]);
    await depositTransaction(depositItems[3]);

    await network.provider.send('hardhat_mine', [ethers.utils.hexValue(1)]);
    await network.provider.send('evm_setAutomine', [true]);

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock + 1,
    );

    expect(accumulator).to.equal(
      '0x88d29f354f12c62a466e92352203fbb6cd5b468d46701d490037a16e5125ea26',
    );
  });

  it('Should return latest accumulator if blockNumber is after last deposit', async function () {
    for (const depositItem of depositItems) {
      await deposit(depositItem);
    }
    const startBlock = Number(await network.provider.send('eth_blockNumber'));

    const accumulator = await validatorAccumulator.findAccumulatorByBlock(
      startBlock + 1,
    );

    expect(accumulator).to.equal(accumulators[accumulators.length - 1]);
  });
});
