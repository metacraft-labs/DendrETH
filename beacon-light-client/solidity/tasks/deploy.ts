import { task } from 'hardhat/config';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';
import {
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

task('deploy', 'Deploy the beacon light client contract')
  .addParam('slot', 'The slot ')
  .addParam('followNetwork', 'The network to follow')
  .setAction(async (args, { run, ethers }) => {
    if (!isSupportedFollowNetwork(args.followNetwork)) {
      logger.warn('This followNetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = await getNetworkConfig(args.followNetwork);

    await run('compile');
    const [deployer] = await ethers.getSigners();

    logger.info(`Deploying contracts with the account: ${deployer.address}`);
    logger.info(`Account balance: ${(await deployer.getBalance()).toString()}`);

    const beaconApi = await getBeaconApi(currentConfig.BEACON_REST_API);

    const beaconLightClient = await (
      await ethers.getContractFactory('BeaconLightClient')
    ).deploy(
      ...(await getConstructorArgs(beaconApi, args.slot, currentConfig)),
    );

    logger.info('>>> Waiting for BeaconLightClient deployment...');

    logger.info(
      `Deploying transaction hash.. ${beaconLightClient.deployTransaction.hash}`,
    );

    const contract = await beaconLightClient.deployed();

    logger.info(`>>> ${contract.address}`);
    logger.info('>>> Done!');
  });
