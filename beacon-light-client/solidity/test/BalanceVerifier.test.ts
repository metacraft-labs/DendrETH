import { Contract } from 'ethers';
import { ethers } from 'hardhat';
import depositContractAbi from './abis/deposit.json';
import { expect } from 'chai';

describe('BalanceVerifier', () => {
  let depositContract: Contract;
  let validatorAccumulator: Contract;
  let verifierMock: Contract;
  let balanceVerifierLido: Contract;
  let balanceVerifierDiva: Contract;

  beforeEach(async () => {
    const owner = (await ethers.getSigners())[5];

    depositContract = await ethers.getContractAt(
      depositContractAbi,
      '0x00000000219ab540356cBB839Cbe05303d7705Fa',
    );

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
      ethers.encodeBytes32String('test test test'),
      1606824023,
      verifierMock.target,
      await owner.getAddress(),
    );

    balanceVerifierLido = await (
      await ethers.getContractFactory('BalanceVerifierLido')
    ).deploy(
      1,
      ethers.encodeBytes32String('test test test'),
      1606824023,
      verifierMock.target,
      await owner.getAddress(),
    );
  });

  it('Should verify successfully', async () => {
    const inputData = [
      ethers.encodeBytes32String('test test test'),
      9135288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await balanceVerifierDiva.verify(...inputData);

    await balanceVerifierLido.verify(...inputData);
  });

  it('Should revert if slot out of range', async () => {
    const inputData = [
      ethers.encodeBytes32String('test test test'),
      9121288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(
      balanceVerifierDiva.verify(...inputData),
    ).to.be.revertedWithCustomError(
      balanceVerifierDiva,
      'BeaconRootOutOfRange',
    );

    await expect(
      balanceVerifierLido.verify(...inputData),
    ).to.be.revertedWithCustomError(
      balanceVerifierLido,
      'BeaconRootOutOfRange',
    );
  });

  it('Should beacon root not found', async () => {
    const inputData = [
      ethers.encodeBytes32String('test test test'),
      9136288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(
      balanceVerifierDiva.verify(...inputData),
    ).to.be.revertedWithCustomError(balanceVerifierDiva, 'NoBlockRootFound');

    await expect(
      balanceVerifierLido.verify(...inputData),
    ).to.be.revertedWithCustomError(balanceVerifierLido, 'NoBlockRootFound');
  });

  it('Should revert when verifier fails', async () => {
    await verifierMock.setSuccess(false);

    const inputData = [
      ethers.encodeBytes32String('test test test'),
      9135288,
      2435,
      0,
      1,
      1,
      2,
    ];

    await expect(
      balanceVerifierDiva.verify(...inputData),
    ).to.be.revertedWithCustomError(balanceVerifierDiva, 'VerificationFailed');

    await expect(
      balanceVerifierLido.verify(...inputData),
    ).to.be.revertedWithCustomError(balanceVerifierLido, 'VerificationFailed');
  });
});
