import { Queue } from 'bullmq';
import { IBeaconApi } from '../../abstraction/beacon-api-interface';
import { getInputFromTo } from './get_light_client_input_from_to';
import { ProofInputType } from '../../types/types';
import { Config } from '../../constants/constants';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

export default async function doUpdate(
  beaconApi: IBeaconApi,
  proofGeneratorQueue: Queue<ProofInputType, any, string>,
  from: number,
  to: number,
  networkConfig: Config,
) {
  console.log('');
  logger.info('Getting Update..');

  const result = await getInputFromTo(from, to, beaconApi, networkConfig);

  // the task will repeat in case something fails
  await proofGeneratorQueue.add('proofGenerate', result, {
    attempts: 10,
    backoff: {
      type: 'fixed',
      delay: 60000,
    },
    priority: from,
  });
  logger.info('Got Update');
  console.log('');
}
