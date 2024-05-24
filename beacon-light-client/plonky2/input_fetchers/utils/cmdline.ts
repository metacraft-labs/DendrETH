import yargs, { Options } from 'yargs';
import { hideBin } from 'yargs/helpers';
import config from '../../common_config.json';

const args = yargs(hideBin(process.argv));

export class CommandLineOptionsBuilder {
  usage(description: string) {
    args.usage(description);
    return this;
  }

  withRedisOpts() {
    args
      .option('redis-host', {
        describe: 'The Redis host',
        type: 'string',
        default: config['redis-host'],
        description: 'Specifies the Redis host address',
      })
      .option('redis-port', {
        describe: 'The Redis port',
        type: 'number',
        default: Number(config['redis-port']),
        description: 'Specifies the Redis port number',
      });

    return this;
  }

  withFileSStorageOps() {
    args.option('folder_name', {
      describe: 'Sets the name of the folder proofs will be stored in',
      type: 'string',
    });

    return this;
  }

  withS3StorageOpts() {
    args
      .option('aws-endpoint-url', {
        describe: 'The aws enpoint url',
        type: 'string',
      })
      .option('aws-region', {
        describe: 'The aws region',
        type: 'string',
      })
      .option('aws-bucket-name', {
        describe: 'The name of the aws bucket',
        type: 'string',
      });

    return this;
  }

  withAzureBlobStorageOpts() {
    args
      .option('azure-account-name', {
        describe: 'Sets the name of the azure account',
        type: 'string',
      })
      .option('azure-container-name', {
        describe: 'Sets the name of the azure container',
        type: 'string',
      });

    return this;
  }

  withProofStorageOpts() {
    return this.option('proof_storage_type', {
      describe: 'Sets the type of proof storage',
      type: 'string',
    })
      .withRedisOpts()
      .withFileSStorageOps()
      .withS3StorageOpts()
      .withAzureBlobStorageOpts();
  }

  withLightCleanOpts() {
    args.option('clean-duration', {
      alias: 'clean-duration',
      describe: 'The time between each clean in ms',
      type: 'number',
      default: 5000,
    });

    return this;
  }

  withProtocolOpts() {
    args.option('protocol', {
      alias: 'protocol',
      describe: 'The protocol',
      type: 'string',
      demandOption: true,
      description: 'Sets the protocol',
    });

    return this;
  }

  withBeaconNodeOpts() {
    args.option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'array',
      default: config['beacon-node'],
      description: 'Sets a custom beacon node url',
    });

    return this;
  }

  withRangeOpts() {
    args
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
      });

    return this;
  }

  option(opt: string, settings: Options) {
    args.option(opt, settings);
    return this;
  }

  build() {
    return args.argv;
  }
}
