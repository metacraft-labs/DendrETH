import { CommandLineOptionsBuilder } from '../cmdline';
import { CommitmentMapperScheduler } from './scheduler';
import config from '../common_config.json';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .withBeaconNodeOpts()
    .option('sync-epoch', {
      describe: 'The sync epoch',
      type: 'number',
      default: undefined,
      description: 'Starts syncing from this epoch',
    })
    .withRangeOpts()
    .option('run-once', {
      describe: 'Should run script for one epoch',
      type: 'boolean',
      default: false,
    })
    .build();

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(options);
  await scheduler.start(options['run-once']);
  await scheduler.dispose();
})();
