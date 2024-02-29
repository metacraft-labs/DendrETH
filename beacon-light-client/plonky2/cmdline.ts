import yargs from "yargs";
import config from "./common_config.json";

export class CommandLineOptionsBuilder {
  usage(description: string) {
    yargs.usage(description);
    return this;
  }

  withRedisOpts() {
    yargs
      .option('redis-host', {
        describe: 'The Redis host',
        type: 'string',
        default: config['redis-host'],
        description: 'Sets a custom redis connection',
      })
      .option('redis-port', {
        describe: 'The Redis port',
        type: 'number',
        default: Number(config['redis-port']),
        description: 'Sets a custom redis connection',
      });
    return this;
  }

  withFileSStorageOps() {
    yargs
      .option('folder_name', {
        describe: 'Sets the name of the folder proofs will be stored in',
        type: 'string',
      });

    return this;
  }

  withS3StorageOpts() {
    yargs
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
    yargs
      .option('azure-account-name', {
        describe: 'Sets the name of the azure account',
        type: 'string',
      })
      .option('azure-container-name', {
        describe: 'Sets the name of the azure container',
        type: 'string',
      })

    return this;

  }

  withProofStorageOpts() {
    return this
      .option('proof_storage_type', {
        describe: 'Sets the type of proof storage',
        type: 'string',
      })
      .withRedisOpts()
      .withFileSStorageOps()
      .withS3StorageOpts()
      .withAzureBlobStorageOpts();
  }

  option(opt: string, settings: any) {
    yargs.option(opt, settings);
    return this;
  }

  build() {
    return yargs.argv;
  }
}
