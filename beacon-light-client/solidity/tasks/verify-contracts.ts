import { task } from 'hardhat/config';
import { getConstructorArgs } from './utils';

task('verify-contracts', 'Verify')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .addParam('slot', 'The slot ')
  .setAction(async (args, { run }) => {
    await run('verify:verify', {
      address: args.lightclient,
      constructorArguments: await getConstructorArgs(args.slot),
    });
  });
