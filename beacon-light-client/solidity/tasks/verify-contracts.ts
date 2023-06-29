import { task } from 'hardhat/config';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';
import * as networkConfig from '../../../relay/constants/network_config.json';
import { Config } from '../../../relay/constants/constants';

task('verify-contracts', 'Verify')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .addParam('slot', 'The slot ')
  .addParam('follownetwork', 'The network to follow')
  .setAction(async (args, { run }) => {
    if (!networkConfig[args.follownetwork]) {
      console.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = networkConfig[args.follownetwork] as Config;

    const beaconApi = new BeaconApi([currentConfig.BEACON_REST_API!]);

    await run('verify:verify', {
      address: args.lightclient,
      constructorArguments: await getConstructorArgs(
        beaconApi,
        args.slot,
        currentConfig,
      ),
    });
  });
