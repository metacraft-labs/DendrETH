import { CommandLineOptionsBuilder } from '../cmdline';
import commonConfig from '../common_config.json';
import { getBalancesInput } from './get_balances_input';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'array',
      default: commonConfig['beacon-node'],
      description: 'Sets a custom beacon node url',
    })
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
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: Infinity,
      description: 'Sets the number of validators to take',
    })
    .option('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    })
    .build();

  await getBalancesInput({
    withdrawCredentials: options['withdraw-credentials'],
    beaconNodeUrls: options['beacon-node'],
    slot: options['slot'],
    take: options['take'],
    offset: options['offset'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
  });
})();
