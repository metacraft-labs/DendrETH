import { task } from 'hardhat/config';
import { getConstructorArgs } from './utils';

task('deploy', 'Deploy the beacon light client contract').setAction(
  async (_, { run, ethers, network }) => {
    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const beaconLightClient = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(...getConstructorArgs(network.name));

    console.log('>>> Waiting for BeaconLightClient deployment...');

    console.log(
      'Deploying transaction hash..',
      beaconLightClient.deployTransaction.hash,
    );

    const contract = await beaconLightClient.deployed();

    console.log(`>>> ${contract.address}`);
    console.log('>>> Done!');
  },
);
