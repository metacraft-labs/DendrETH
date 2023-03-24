import { task } from 'hardhat/config';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { getConstructorArgs } from './utils';

task('verify-contracts', 'Verify')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .addParam('slot', 'The slot ')
  .setAction(async (args, { run }) => {
    const config = {
      BEACON_REST_API: process.env.BEACON_REST_API,
    };

    checkConfig(config);

    const beaconApi = new BeaconApi(config.BEACON_REST_API!);

    await run('verify:verify', {
      address: args.lightclient,
      constructorArguments: await getConstructorArgs(beaconApi, args.slot),
    });
  });
