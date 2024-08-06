import { Queue } from 'bullmq';
import { GetUpdate } from '@/types/types';
import { Config } from '@/constants/constants';
import { computeSyncCommitteePeriodAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { IBeaconApi } from '@/abstraction/beacon-api-interface';
import { findClosestValidBlock } from '@/workers/poll-updates/get_light_client_input_from_to';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

export async function addUpdate(
  optimisticSlot: number,
  slotsJump: number,
  headSlot: number,
  updateQueue: Queue<GetUpdate>,
  networkConfig: Config,
  beaconApi: IBeaconApi,
): Promise<boolean> {
  const jobsInQueueSortedByFrom = (await updateQueue.getJobs()).sort(
    (a, b) => a.data.from - b.data.from,
  );

  for (let i = 0; i < jobsInQueueSortedByFrom.length; i++) {
    // skip failed jobs
    if (await jobsInQueueSortedByFrom[i].isFailed()) continue;

    if (jobsInQueueSortedByFrom[i].data.from === optimisticSlot) {
      optimisticSlot = jobsInQueueSortedByFrom[i].data.to;
    }
  }

  const nextSlot = await getNextSlot(
    optimisticSlot,
    slotsJump,
    headSlot,
    beaconApi,
  );

  if (optimisticSlot >= nextSlot) {
    logger.info('No new enough slots');
    return false;
  }

  logger.info('New update from-to added');

  await updateQueue.add(
    'update',
    {
      from: optimisticSlot,
      to: nextSlot,
      networkConfig: networkConfig,
    },
    {
      attempts: 10,
      backoff: {
        type: 'fixed',
        delay: 15000,
      },
      priority:
        Math.round(optimisticSlot / 2000000) + (optimisticSlot % 2000000),
    },
  );

  return true;
}

async function getNextSlot(
  slot: number,
  slotsJump: number,
  headSlot: number,
  beaconApi: IBeaconApi,
) {
  const slotsPerPeriod = await beaconApi.getSlotsPerSyncCommitteePeriod();
  const slotsPerEpoch = await beaconApi.getSlotsPerEpoch();
  const periodAtSlot = computeSyncCommitteePeriodAt(
    BigInt(slot),
    slotsPerPeriod,
  );
  const periodAtHeadSlot = computeSyncCommitteePeriodAt(
    BigInt(headSlot),
    slotsPerPeriod,
  );

  console.log('period at slot', periodAtSlot);
  console.log('period at head slot', periodAtHeadSlot);

  if (periodAtSlot + 1n >= periodAtHeadSlot) {
    // next slot will be the closest multiple of slotsJump to headSlot
    const potentialNewSlot = headSlot - (headSlot % slotsJump);

    const result = await findClosestValidBlock(
      potentialNewSlot,
      beaconApi,
      headSlot,
    );

    return result.nextBlockHeader.slot;
  }

  // next slot will be the first slot of the last epoch of the next period
  const potentialNewSlot =
    BigInt(periodAtSlot + 1n) * slotsPerPeriod +
    (slotsPerPeriod - slotsPerEpoch);

  console.log('potential new slot', potentialNewSlot);

  console.log('head slot', headSlot);

  const result = await findClosestValidBlock(
    Number(potentialNewSlot),
    beaconApi,
    headSlot,
  );

  return result.nextBlockHeader.slot;
}
