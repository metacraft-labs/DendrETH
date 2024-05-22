import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import commonConfig from '../../../../common_config.json';
import { getBalancesInput } from '../lib/scheduler';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .withBeaconNodeOpts()
    .option('slot', {
      alias: 'slot',
      describe: 'The state slot',
      type: 'number',
      default: undefined,
      description: 'Fetches the balances for this slot',
    })
    .option('withdraw-credentials', {
      alias: 'withdraw-credentials',
      describe: 'The withdrawal credentials',
      type: 'string',
      demandOption: true,
      description: 'Sets the withdrawal credentials',
    })
    .withRangeOpts()
    .withProtocolOpts()
    .build();

  await getBalancesInput({
    withdrawCredentials: options['withdraw-credentials'],
    beaconNodeUrls: options['beacon-node'],
    slot: options['slot'],
    take: options['take'],
    offset: options['offset'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
    protocol: options['protocol'],
  });
})();
