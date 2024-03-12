import { CommandLineOptionsBuilder } from '../cmdline';
import { CommitmentMapperScheduler } from './scheduler';
import config from '../common_config.json';

(async () => {
  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .option('beacon-node', {
      describe: 'The beacon node url',
      type: 'array',
      default: config['beacon-node'],
      description: 'Sets a custom beacon node url',
    })
    .option('sync-epoch', {
      describe: 'The sync epoch',
      type: 'number',
      default: undefined,
      description: 'Starts syncing from this epoch',
    })
    .option('offset', {
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    })
    .option('take', {
      describe: 'The number of validators to take',
      type: 'number',
      default: Infinity,
      description: 'Sets the number of validators to take',
    })
    .option('mock', {
      describe: 'Runs the tool without doing actual calculations',
      type: 'boolean',
      default: false,
      description: 'Runs the tool without doing actual calculations.',
    })
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
