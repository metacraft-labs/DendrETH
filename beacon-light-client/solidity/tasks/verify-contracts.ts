import { task } from 'hardhat/config';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';
import {
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

task('verify-contracts', 'Verify')
  .addParam('lightClient', 'The address of the BeaconLightClient contract')
  .addParam('slot', 'The slot ')
  .addParam('followNetwork', 'The network to follow')
  .setAction(async (args, { run }) => {
    if (!isSupportedFollowNetwork(args.followNetwork)) {
      logger.warn('This followNetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = await getNetworkConfig(args.followNetwork);

    const beaconApi = await getBeaconApi(currentConfig.BEACON_REST_API!);

    console.log(await getConstructorArgs(beaconApi, args.slot, currentConfig));

    await run('verify:verify', {
      address: args.lightClient,
      constructorArguments: await getConstructorArgs(
        beaconApi,
        args.slot,
        currentConfig,
      ),
    });
  });
