import yargs from 'yargs';
import config from "../common_config.json";

import * as utils from "./utils";

(async () => {
  const options = yargs
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .option('redis-host ', {
      alias: 'redis-host',
      describe: 'The Redis host',
      type: 'string',
      default: config['redis-host'],
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: Number(config['redis-port']),
      description: 'Sets a custom redis connection',
    })
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: config['beacon-node'],
      description: 'Sets a custom beacon node url',
    })
    .option('sync-epoch', {
      alias: 'sync-epoch',
      describe: 'The sync epoch',
      type: 'number',
      default: undefined,
      description: 'Starts syncing from this epoch',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: undefined,
      description: 'Sets the number of validators to take',
    })
    .option('run-once', {
      alias: 'run-once',
      describe: 'Should run script for one epoch',
      type: 'boolean',
      default: false,
    })
    .argv;

  const scheduler = new utils.CommitmentMapperScheduler();
  await scheduler.init(options);
  await scheduler.start(options['run-once']);
  await scheduler[Symbol.asyncDispose]();
})();
