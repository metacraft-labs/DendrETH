import { Queue } from 'bullmq';
import { IBeaconApi } from '../../abstraction/beacon-api-interface';
import { getInputFromTo } from './get_light_client_input_from_to';
import { ProofInputType } from '../../types/types';
import { Config, PROOF_GENERATOR_QUEUE } from '../../constants/constants';
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
  await proofGeneratorQueue.add(PROOF_GENERATOR_QUEUE, result, {
    attempts: 10,
    backoff: {
      type: 'fixed',
      delay: 60000,
    },
    priority: Math.round(from / 2000000) + (from % 2000000),
  });
  logger.info('Got Update');
  console.log('');
}
