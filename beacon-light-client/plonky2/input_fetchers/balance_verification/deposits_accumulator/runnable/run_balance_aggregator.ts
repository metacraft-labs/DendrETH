import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import { storeBalanceVerificationData } from '../lib/get_balance_verification_data';

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
    .option('rpc-url', {
      describe: 'The RPC URL',
      type: 'string',
      default: 'http://127.0.0.1:8545/',
    })
    .option('address', {
      describe: 'The validators accumulator contract address',
      type: 'string',
      default: undefined,
    })
    .option('sync-block', {
      describe: 'The starting block to fetch events from',
      type: 'number',
      default: false,
    })
    .withRangeOpts()
    .withProtocolOpts()
    .build();

  await storeBalanceVerificationData({
    beaconNodeUrls: options['beacon-node'],
    slot: options['slot'],
    take: options['take'],
    offset: options['offset'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
    address: options['address'],
    rpcUrl: options['rpc-url'],
    syncBlock: options['sync-block'],
    protocol: options['protocol'],
  });
})();
