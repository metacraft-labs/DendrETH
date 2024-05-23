import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import { DepositScheduler } from '../lib/deposit_scheduler';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .withBeaconNodeOpts()
    .option('rpc-url', {
      describe: 'The RPC URL',
      type: 'string',
      default: 'http://127.0.0.1:8545/',
    })
    .option('sync-block', {
      describe: 'The starting block to fetch events from',
      type: 'number',
      default: undefined,
    })
    .option('address', {
      describe: 'The validators accumulator contract address',
      type: 'string',
      default: undefined,
    })
    .build();

  const scheduler = new DepositScheduler();
  await scheduler.init(options);
  await scheduler.start();
  await scheduler.dispose();
})();
