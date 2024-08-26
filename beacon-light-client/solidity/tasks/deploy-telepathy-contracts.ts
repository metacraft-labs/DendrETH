import { task } from 'hardhat/config';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { getConstructorArgs, getTelepathyConstructorArgs } from './utils';
import {
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

task('deploy-telepathy', 'Deploy the telepathy light client contract')
  .addParam('slot', 'The slot you want to deploy the Telepathy contract')
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

    const getConstructorArgs = await getTelepathyConstructorArgs(
      beaconApi,
      args.slot,
    );

    console.log('Constructor args:', ...getConstructorArgs);

    const telepathyLightClient = await (
      await ethers.getContractFactory('TelepathyLightClient')
    ).deploy(...getConstructorArgs);

    logger.info('>>> Waiting for TelepathyLightClient deployment...');

    logger.info(
      `Deploying transaction hash.. ${telepathyLightClient.deployTransaction.hash}`,
    );

    const contract = await telepathyLightClient.deployed();

    logger.info(`>>> ${contract.address}`);
    logger.info('>>> Done!');
  });
