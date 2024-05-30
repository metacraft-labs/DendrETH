import { Contract } from 'ethers';
import { task } from 'hardhat/config';
const depositItems = [
  {
    pubkey:
      '0x89bcf22c91a560d95d09c1192664eea1baab0780b6d4441ca39d1cb5094b177b17f47a67b16fb972bfd3b78b602ffeee',
    withdrawalCredentials:
      '0x0100000000000000000000008ead0e8fec8319fd62c8508c71a90296efd4f042',
    signature:
      '0xaf92ccc88c4b1eca2f7dffb7c9288c014b2dc358d4846037a71f22a7ebab387795fd88fd29ab6304e25021fae7d99e320b8f9cbf6a5809a9b61e6612a2c838cea8f90a2e90172f111d17c429215d61452ee341ab17915c415696531ff9a69fe8',
    depositDataRoot:
      '0x757b092b9157a2a946ebf5209660433a71ccabe15b50dbf0cfcc21b9f090e1b5',
  },
  {
    pubkey:
      '0xb781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73',
    withdrawalCredentials:
      '0x00ea361fd66a174b289f0b26ed7bbcaabdec4b7d47d5527ff72a994c9c1c156f',
    signature:
      '0xb735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275',
    depositDataRoot:
      '0xfb798ea86cbeaccf78068557dcefe32f309d37c3f65d531d2374050549d0436c',
  },
];

task('deploy-accumulator', 'Deploy the validators accumulator contract')
  .addOptionalParam('contractAddress', 'The address of the contract')
  .setAction(async (args, { run, ethers }) => {
    run('compile');

    const signer = await ethers.getSigner(
      '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266',
    );

    let validatorAccumulator: Contract;
    if (!args.contractAddress) {
      const contractFactory = await ethers.getContractFactory(
        'ValidatorsAccumulator',
      );

      console.log('Deploying validator accumulator contract');
      validatorAccumulator = await contractFactory
        .connect(signer)
        .deploy('0x00000000219ab540356cBB839Cbe05303d7705Fa', {
          gasLimit: 8000000,
          maxFeePerGas: 16224155402,
        });

      await validatorAccumulator.deployed();

      console.log('depositing validators');
      for (const depositItem of depositItems.slice(1)) {
        const tx = await validatorAccumulator.deposit(
          depositItem.pubkey,
          depositItem.withdrawalCredentials,
          depositItem.signature,
          depositItem.depositDataRoot,
          { value: ethers.utils.parseEther('32').toString() },
        );
        console.log(await validatorAccumulator.getValidatorsAccumulator());
        await tx.wait();
      }
    } else {
      validatorAccumulator = await ethers.getContractAt(
        'ValidatorsAccumulator',
        args.contractAddress,
        signer,
      );

      console.log('depositing validators');
      for (const depositItem of depositItems.slice(0, 1)) {
        const tx = await validatorAccumulator.deposit(
          depositItem.pubkey,
          depositItem.withdrawalCredentials,
          depositItem.signature,
          depositItem.depositDataRoot,
          { value: ethers.utils.parseEther('32').toString() },
        );
        await tx.wait();
      }
    }

    console.log('address:', validatorAccumulator.address);
  });
