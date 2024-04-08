import { Config } from '../../constants/constants';
import { getProofInput } from './get_ligth_client_input';
import { IBeaconApi } from '../../abstraction/beacon-api-interface';
import { SyncAggregate } from '../../types/types';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { prometheusTiming } from '@dendreth/utils/ts-utils/prometheus-utils';
import { BeaconBlockHeader } from '@lodestar/types/phase0';

const logger = getGenericLogger();

export async function getInputFromTo(
  from: number,
  to: number,
  beaconApi: IBeaconApi,
  networkConfig: Config,
) {
  const prevBlockHeader = await beaconApi.getExistingBlockHeader(from);

  const headSlot = await beaconApi.getCurrentHeadSlot();

  let { signature_slot, nextBlockHeader, sync_aggregate } =
    await prometheusTiming(
      async () => await findClosestValidBlock(to, beaconApi, headSlot),
      'findClosestValidBlock',
    );

  logger.info('Getting prevBlockHeaderStateInfo..');
  const {
    finalityHeader: prevFinalizedHeader,
    finalityHeaderBranch: prevFinalityBranch,
    syncCommittee,
    syncCommitteeBranch,
  } = await beaconApi.getPrevBlockHeaderStateInfo(from, signature_slot);

  logger.info('Getting finalityBlockAndProof..');
  const {
    finalityHeader: finalizedHeader,
    finalityHeaderBranch: finalityBranch,
  } = await prometheusTiming(
    async () => await beaconApi.getFinalityBlockAndProof(nextBlockHeader.slot),
    'getFinalityBlockAndProof',
  );

  logger.info('Getting getBlockExecutionPayloadAndProof..');
  const {
    executionPayloadHeader: executionPayload,
    executionPayloadBranch: finalizedHeaderExecutionBranch,
  } = await prometheusTiming(
    async () =>
      await beaconApi.getBlockExecutionPayloadAndProof(finalizedHeader.slot),
    'getBlockExecutionPayloadAndProof',
  );

  return {
    proofInput: await getProofInput({
      prevBlockHeader,
      nextBlockHeader,
      prevFinalizedHeader,
      syncCommitteeBranch,
      syncCommittee,
      config: networkConfig,
      prevFinalityBranch,
      signature_slot: signature_slot,
      finalizedHeader,
      finalityBranch,
      executionPayload,
      finalizedHeaderExecutionBranch,
      sync_aggregate,
    }),
    prevUpdateSlot: prevBlockHeader.slot,
    updateSlot: nextBlockHeader.slot,
  };
}

export async function findClosestValidBlock(
  to: number,
  beaconApi: IBeaconApi,
  headSlot: number,
): Promise<{
  signature_slot: number;
  nextBlockHeader: BeaconBlockHeader;
  sync_aggregate: SyncAggregate;
}> {
  let nextBlockHeader;
  let sync_aggregate;
  let signature_slot;
  let nextHeaderSlotSearchIndex = to;

  while (true) {
    logger.info('Getting BlockHeader from BeaconApi..');
    nextBlockHeader = await prometheusTiming(
      async () =>
        await beaconApi.getBlockHeaderOrClosestExisting(
          nextHeaderSlotSearchIndex,
          headSlot,
        ),
      'getFinalityBlockAndProof',
    );

    logger.info('Getting SyncAggregate from BeaconApi..');
    const syncAggregateResult =
      await beaconApi.getBlockSyncAggregateOrClosestExisting(
        nextBlockHeader.slot + 1,
        headSlot,
      );

    const length = BigInt(
      syncAggregateResult.sync_aggregate.sync_committee_bits,
    )
      .toString(2)
      .split('')
      .filter(x => x == '1').length;

    if (length * 3 > 1024) {
      // x - 2/3 * 512 === 3*x - 1024
      logger.info(`Found block with enough signers (${length})..`);
      sync_aggregate = syncAggregateResult.sync_aggregate;
      signature_slot = syncAggregateResult.slot;
      break;
    }

    // this is the next available slot after the nextBlockHeader slot which was not signed by enough validators
    logger.info(`Not enough signers in committee (${length})..`);
    nextHeaderSlotSearchIndex = syncAggregateResult.slot;
  }

  return { signature_slot, nextBlockHeader, sync_aggregate };
}
