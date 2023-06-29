import { task } from 'hardhat/config';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';
import * as networkConfig from '../../../relay/constants/network_config.json';
import { Config } from '../../../relay/constants/constants';

task('deploy', 'Deploy the beacon light client contract')
  .addParam('slot', 'The slot ')
  .addParam('follownetwork', 'The network to follow')
  .setAction(async (args, { run, ethers, network }) => {
    if (!networkConfig[args.follownetwork]) {
      console.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = networkConfig[args.follownetwork] as Config;

    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const beaconApi = new BeaconApi([currentConfig.BEACON_REST_API]);

    const beaconLightClient = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(
      ...(await getConstructorArgs(beaconApi, args.slot, currentConfig)),
    );

    console.log('>>> Waiting for BeaconLightClient deployment...');

    console.log(
      'Deploying transaction hash..',
      beaconLightClient.deployTransaction.hash,
    );

    const contract = await beaconLightClient.deployed();

    console.log(`>>> ${contract.address}`);
    console.log('>>> Done!');
  });
