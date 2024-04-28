import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import { CommitmentMapperScheduler } from '../lib/scheduler';
(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .withBeaconNodeOpts()
    .withRangeOpts()
    .option('sync-slot', {
      describe: 'The sync slot',
      type: 'number',
      default: undefined,
      description: 'Starts syncing from this slot',
    })
    .option('run-once', {
      describe: 'Should run script for one slot',
      type: 'boolean',
      default: false,
    })
    .build();

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(options);
  await scheduler.start(options['run-once']);
  await scheduler.dispose();
})();
