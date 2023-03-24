import { task } from 'hardhat/config';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';

task('deploy', 'Deploy the beacon light client contract')
  .addParam('slot', 'The slot ')
  .setAction(async (args, { run, ethers, network }) => {
    const config = {
      BEACON_REST_API: process.env.BEACON_REST_API,
    };

    checkConfig(config);

    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const beaconApi = new BeaconApi(config.BEACON_REST_API!);

    const beaconLightClient = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(...(await getConstructorArgs(beaconApi, args.slot)));

    console.log('>>> Waiting for BeaconLightClient deployment...');

    console.log(
      'Deploying transaction hash..',
      beaconLightClient.deployTransaction.hash,
    );

    const contract = await beaconLightClient.deployed();

    console.log(`>>> ${contract.address}`);
    console.log('>>> Done!');
  });
