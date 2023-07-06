import { Queue } from 'bullmq';
import { IBeaconApi } from '../../abstraction/beacon-api-interface';
import { IRedis } from '../../abstraction/redis-interface';
import {
  findClosestValidBlock,
  getInputFromTo,
} from './get_light_client_input_from_to';
import { ProofInputType } from '../../types/types';
import { Config } from '../../constants/constants';
import { computeSyncCommitteePeriodAt } from '../../../libs/typescript/ts-utils/ssz-utils';

export default async function doUpdate(
  redis: IRedis,
  beaconApi: IBeaconApi,
  proofGeneratorQueue: Queue<ProofInputType, any, string>,
  lastDownloadedUpdateKey: string | undefined,
  from: number | undefined,
  slotsJump: number,
  networkConfig: Config,
) {
  const currentHeadSlot = await beaconApi.getCurrentHeadSlot();

  const lastDownloadedUpdate = lastDownloadedUpdateKey
    ? Number.parseInt((await redis.get(lastDownloadedUpdateKey))!)
    : from!;

  console.log('Last downloaded update: ', lastDownloadedUpdate);

  const initialUpdate =
    lastDownloadedUpdate - (lastDownloadedUpdate % slotsJump);

  if (currentHeadSlot <= initialUpdate + slotsJump) {
    console.log('No new enought slot');
    // the job will be retried
    throw new Error('No new enought slot');
  }

  let nextHeaderSlot = initialUpdate + slotsJump;


  console.log('Next supposed header', nextHeaderSlot);

  // JUMP to the next closest to the present header
  while (
    nextHeaderSlot + slotsJump < currentHeadSlot &&
    computeSyncCommitteePeriodAt(nextHeaderSlot + 32 + slotsJump) <=
      computeSyncCommitteePeriodAt(lastDownloadedUpdate) + 1
  ) {
    nextHeaderSlot = nextHeaderSlot + slotsJump;
  }

  console.log('Actuall next header', nextHeaderSlot);

  const result = await getInputFromTo(
    lastDownloadedUpdate,
    nextHeaderSlot,
    currentHeadSlot,
    beaconApi,
    networkConfig,
  );

  // the task will repeat in case something fails
  await proofGeneratorQueue.add('proofGenerate', result, {
    attempts: 10,
    backoff: {
      type: 'fixed',
      delay: 60000,
    },
  });

  if (lastDownloadedUpdateKey) {
    await redis.set(lastDownloadedUpdateKey, result.updateSlot);
  }
}
