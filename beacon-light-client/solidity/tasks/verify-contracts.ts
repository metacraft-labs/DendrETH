import { task } from 'hardhat/config';
import { getConstructorArgs } from './utils';

task('verify-contracts', 'Verify')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .setAction(async (args, { run, network }) => {
    await run('verify:verify', {
      address: args.lightclient,
      constructorArguments: await getConstructorArgs(
        (network.config as any).beaconApi,
        23,
      ),
    });
  });
