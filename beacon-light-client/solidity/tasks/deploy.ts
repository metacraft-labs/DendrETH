import { task } from 'hardhat/config';
import { getConstructorArgs } from './utils';

task('deploy', 'Deploy the beacon light client contract').addParam("targetNetwork", "Network to use when updating the light client.", "mainnet").setAction(
  async (taskArgs, { run, ethers, network }) => {
    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const beaconLightClient = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(...getConstructorArgs(taskArgs.targetNetwork));

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
