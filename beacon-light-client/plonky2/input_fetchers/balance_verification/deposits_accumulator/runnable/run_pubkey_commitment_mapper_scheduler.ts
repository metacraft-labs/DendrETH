import {
  createSchedulerContext,
  destroySchedulerContext,
  pollEvents,
  purgePubkeyCommitmentMapperData,
  rebuildCommitmentMapperTree,
} from '../lib/pubkey_commitment_mapper_scheduler';
import validatorsAccumulatorAbi from '../../../abi/validators_accumulator_abi.json';

import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import config from '../../../../common_config.json';

config satisfies CommonConfig;

(async () => {
  const cmdlineOpts = new CommandLineOptionsBuilder()
    .option('rebuild', {
      describe: 'Rebuild pubkey commitment mapper tree',
      type: 'boolean',
    })
    .option('purge', {
      describe: 'Delete pubkey commitment mapper data',
      type: 'boolean',
    })
    .option('protocol', {
      describe: 'The protocol to run the commitment mapper for',
      type: 'string',
      demandOption: true,
    })
    .option('contract-address', {
      describe: 'The address of the contract',
      type: 'string',
      demandOption: true,
    })
    .option('json-rpc', {
      describe: 'The address of the Ethereum json rpc',
      type: 'string',
      demandOption: true,
    })
    .build();

  const context = createSchedulerContext({
    redisHost: config['redis-host'],
    redisPort: config['redis-port'],
    ethJsonRPCProviderURL: cmdlineOpts['json-rpc'],
    contractAddress: cmdlineOpts['contract-address'],
    contractAbi: validatorsAccumulatorAbi,
    protocol: cmdlineOpts['protocol'],
  });

  if (cmdlineOpts['purge']) {
    await purgePubkeyCommitmentMapperData(context);
    console.log('Data purged');
    await destroySchedulerContext(context);
    return;
  }

  if (cmdlineOpts['rebuild']) {
    await rebuildCommitmentMapperTree(context);
  }

  await pollEvents(context);
})();
