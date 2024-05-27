import chaiAsPromised from 'chai-as-promised';
import { expect, use } from 'chai';
import { Contract } from 'ethers';
import { ethers, network } from 'hardhat';
import depositItems from './utils/depositData.json';

use(chaiAsPromised);

describe('BalanceVerifier', () => {
  let validatorAccumulator: Contract;
  let verifierMock: Contract;
  let balanceVerifierLido: Contract;
  let balanceVerifierDiva: Contract;

  beforeEach(async () => {
    const owner = (await ethers.getSigners())[5];

    const contractFactory = await ethers.getContractFactory(
      'ValidatorsAccumulator',
    );
    validatorAccumulator = await contractFactory.deploy(
      '0x00000000219ab540356cBB839Cbe05303d7705Fa',
    );

    verifierMock = await (
      await ethers.getContractFactory('VerifierMock')
    ).deploy();

    balanceVerifierDiva = await (
      await ethers.getContractFactory('BalanceVerifierDiva')
    ).deploy(
      1,
      1606824023,
      0x00000000,
      verifierMock.address,
      validatorAccumulator.address,
      owner.address,
    );

    balanceVerifierLido = await (
      await ethers.getContractFactory('BalanceVerifierLido')
    ).deploy(
      1,
      ethers.utils.formatBytes32String('test test test'),
      1606824023,
      verifierMock.address,
      owner.address,
    );
  });

  it('Should verify successfully', async () => {
    const inputData = [
      ethers.utils.formatBytes32String('test test test'),
      9135288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await balanceVerifierLido.verify(...inputData);

    inputData.splice(
      2,
      0,
      Number(await network.provider.send('eth_blockNumber')),
    );
    await balanceVerifierDiva.verify(...inputData);
  });

  it('Should revert if slot out of range', async () => {
    const inputData = [
      ethers.utils.formatBytes32String('test test test'),
      9121288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(balanceVerifierLido.verify(...inputData)).to.be.rejectedWith(
      'BeaconRootOutOfRange',
    );

    inputData.splice(
      2,
      0,
      Number(await network.provider.send('eth_blockNumber')),
    );

    await expect(balanceVerifierDiva.verify(...inputData)).to.be.rejectedWith(
      'BeaconRootOutOfRange',
    );
  });

  it('Should beacon root not found', async () => {
    const inputData = [
      ethers.utils.formatBytes32String('test test test'),
      9136288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(balanceVerifierLido.verify(...inputData)).to.be.rejectedWith(
      'NoBlockRootFound',
    );

    inputData.splice(
      2,
      0,
      Number(await network.provider.send('eth_blockNumber')),
    );

    await expect(balanceVerifierDiva.verify(...inputData)).to.be.rejectedWith(
      'NoBlockRootFound',
    );
  });

  it('Should revert when verifier fails', async () => {
    await verifierMock.setSuccess(false);

    const inputData = [
      ethers.utils.formatBytes32String('test test test'),
      9135288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(balanceVerifierLido.verify(...inputData)).to.be.rejectedWith(
      'VerificationFailed',
    );

    inputData.splice(
      2,
      0,
      Number(await network.provider.send('eth_blockNumber')),
    );

    await expect(balanceVerifierDiva.verify(...inputData)).to.be.rejectedWith(
      'VerificationFailed',
    );
  });

  describe('Diva deposit accumulator', async () => {
    let startBlock: number;

    beforeEach(async () => {
      startBlock = Number(await ethers.provider.getBlockNumber()) + 1;

      for (const depositItem of depositItems) {
        await (
          await validatorAccumulator.deposit(
            depositItem.pubkey,
            depositItem.withdrawalCredentials,
            depositItem.signature,
            depositItem.depositDataRoot,
            { value: ethers.utils.parseEther('32').toString() },
          )
        ).wait();
      }
    });

    it('Should verify the balance of the depositors', async () => {
      const inputData = [
        ethers.utils.formatBytes32String('test test test'),
        9135288,
        startBlock + 15,
        2435,
        0,
        1,
        1,
        2,
      ];

      await balanceVerifierDiva.verify(...inputData);
    });
  });
});
