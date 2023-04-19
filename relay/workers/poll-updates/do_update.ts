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
  lastDownloadedUpdateKey: string,
  slotsJump: number,
  networkConfig: Config,
) {
  const currentHeadSlot = await beaconApi.getCurrentHeadSlot();

  const lastDownloadedUpdate = Number.parseInt(
    (await redis.get(lastDownloadedUpdateKey))!,
  );

  if (currentHeadSlot <= lastDownloadedUpdate + slotsJump) {
    console.log('No new enought slot');
    // the job will be retried
    throw new Error('No new enought slot');
  }


  let nextHeaderSlot = lastDownloadedUpdate + slotsJump;

  // JUMP to the next closest to the present header
  while (
    nextHeaderSlot + slotsJump < currentHeadSlot &&
    computeSyncCommitteePeriodAt(nextHeaderSlot) <=
      computeSyncCommitteePeriodAt(lastDownloadedUpdate) + 1
  ) {
    const { nextBlockHeader } = await findClosestValidBlock(
      nextHeaderSlot,
      beaconApi,
      currentHeadSlot,
    );

    nextHeaderSlot = nextBlockHeader.slot + slotsJump;
  }

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

  await redis.set(lastDownloadedUpdateKey, result.updateSlot);
}
