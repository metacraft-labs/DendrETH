import { task } from 'hardhat/config';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';

task('deploy', 'Deploy the beacon light client contract')
  .addParam('slot', 'The slot ')
  .addParam('follownetwork', 'The network to follow')
  .setAction(async (args, { run, ethers }) => {
    if (args.follownetwork !== 'pratter' && args.follownetwork !== 'mainnet') {
      console.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = getNetworkConfig(args.follownetwork);

    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const beaconApi = new BeaconApi(currentConfig.BEACON_REST_API);

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
