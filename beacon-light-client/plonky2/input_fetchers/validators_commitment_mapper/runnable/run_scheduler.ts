import { CommandLineOptionsBuilder } from '../../utils/cmdline';
import { CommitmentMapperScheduler } from '../lib/scheduler';
(async () => {
  const options = new CommandLineOptionsBuilder()
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
    .option('rebase', {
      describe: 'The slot to rebase onto',
      type: 'number',
      default: undefined,
    })
    .option('recompute', {
      describe: 'The slot to recompute',
      type: 'number',
      default: undefined,
    })
    .build();

  const scheduler = new CommitmentMapperScheduler();
  await scheduler.init(options);
  await scheduler.start({
    runOnce: options['run-once'],
    rebase: options['rebase'],
    recompute: options['recompute'],
  });
  await scheduler.dispose();
})();
