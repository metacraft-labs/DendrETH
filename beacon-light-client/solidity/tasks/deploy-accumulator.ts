import { Contract } from 'ethers';
import { task } from 'hardhat/config';
const depositItems = [
  {
    pubkey:
      '0xaaf6c1251e73fb600624937760fef218aace5b253bf068ed45398aeb29d821e4d2899343ddcbbe37cb3f6cf500dff26c',
    withdrawalCredentials:
      '0x0048281f02e108ec495e48a25d2adb4732df75bf5750c060ff31c864c053d28d',
    signature:
      '0xa89660de6049478c8459383235b8ffaf8be15fd16fc93ea65db6e51e80eaba9c291a9a1bda98d08740a89b3226e89b9804c0f1760e4483d26d33b465fc13565e77a8430900ca245a2b628546abfa6172c758ab81d23bd360bb233a2771242f8f',
    depositDataRoot:
      '0xfce4b23c858a57df1af11aa3fec24b4d50e8048366aa88fbf2bf8b6c6f877b71',
  },
  {
    pubkey:
      '0x8aa5bbee21e98c7b9e7a4c8ea45aa99f89e22992fa4fc2d73869d77da4cc8a05b25b61931ff521986677dd7f7159e8e6',
    withdrawalCredentials:
      '0x0034432b9814c1972e63d60d763584ceea8542de8d397120136e60d731b2de31',
    signature:
      '0x961ef689a27416b35ef02420ce1d11fddbb4c31c701859783406d3cfcc5abee5b3a2d3aafa7ce6289006c5f3621917d104b6a32b6d403d3b6134518448d1eb36751f5577c890ccfd839744b4cf644b88996ef7fccaa6e6dab8e2fafd6d4b41d4',
    depositDataRoot:
      '0x691cbabdcebb83c73543998abd4ecba8a956003b6faa966e39a5e8109a21f524',
  },
  {
    pubkey:
      '0x996323af7e545fb6363ace53f1538c7ddc3eb0d985b2479da3ee4ace10cbc393b518bf02d1a2ddb2f5bdf09b473933ea',
    withdrawalCredentials:
      '0x00a1b4f86e8c047b4ff3d13b5f4920930e95dae0381d93fed0fa5e6dd7c1076a',
    signature:
      '0xb337e70a8e5831b18e0df808a64df57a41c2b3349db8e0e3ebdaab2b54e94bf4bcc8287ca98bed3c6c287d188b41186a074c57497beba90dcd261083b429ea9a58b2f117d2f6ca6e91a653cc98f8d216d3a26dd34c11c5487b1a4985b32030e7',
    depositDataRoot:
      '0xa25b1cf45e2d20168307d6f6fd8d74535b745328d9df49fbec79d89ebc923b3b',
  },
  {
    pubkey:
      '0xa1584dfe1573df8ec88c7b74d76726b4821bfe84bf886dd3c0e3f74c2ea18aa62ca44c871fb1c63971fccf6937e6501f',
    withdrawalCredentials:
      '0x007bc82d3f05f6fe1f0b6ae2505f7170108de13be1d3c1b374e19c64403b659a',
    signature:
      '0xa6d0d970d645c9ff937e9b695ea2a7edbbbb69c0694fca3289b59a4ecdd7982ac057153c8b4cf23383aa0bea60cb07830cac2e15000b028bb76f705b51263db25e2fd52657025411e24f442e5f6646fae4ab6d9393b0b5810244d815fff8213f',
    depositDataRoot:
      '0xa19eaa4ba8ac4fd7ec0fc159cc1d9dcda88031c8abdc8ef51acb7171071150ef',
  },
];

task('deploy-accumulator', 'Deploy the validators accumulator contract')
  .addOptionalParam('contractAddress', 'The address of the contract')
  .setAction(async (args, { run, ethers }) => {
    run('compile');

    const signer = (await ethers.getSigners())[0];

    let validatorAccumulator: Contract;
    if (!args.contractAddress) {
      const contractFactory = await ethers.getContractFactory(
        'ValidatorsAccumulator',
      );

      console.log('Deploying validator accumulator contract');
      validatorAccumulator = await contractFactory
        .connect(signer)
        .deploy('0x4242424242424242424242424242424242424242');

      await validatorAccumulator.deployed();

      console.log('depositing validators');
      for (const depositItem of depositItems) {
        const tx = await validatorAccumulator.deposit(
          depositItem.pubkey,
          depositItem.withdrawalCredentials,
          depositItem.signature,
          depositItem.depositDataRoot,
          {
            value: ethers.utils.parseEther('32').toString(),
          },
        );
        await tx.wait();
        console.log(
          'accumulator',
          await validatorAccumulator.getValidatorsAccumulator(),
        );
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
