import yargs, { Options } from 'yargs';
import { hideBin } from 'yargs/helpers';
import config from './common_config.json';

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

  option(opt: string, settings: Options) {
    args.option(opt, settings);
    return this;
  }

  build() {
    return args.argv;
  }
}
