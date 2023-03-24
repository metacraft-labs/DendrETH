import { PRATER } from '../../constants/constants';
import { getProofInput } from './get_ligth_client_input';
import { IBeaconApi } from '../../abstraction/beacon-api-interface';

export async function getInputFromTo(
  from: number,
  to: number,
  headSlot: number,
  beaconApi: IBeaconApi,
) {
  const prevBlockHeader = await beaconApi.getExistingBlockHeader(from);

  let nextBlockHeader;
  let sync_aggregate;
  let signature_slot;
  let nextHeaderSlotSearchIndex = to;

  while (true) {
    nextBlockHeader = await beaconApi.getBlockHeaderOrClosestExisting(
      nextHeaderSlotSearchIndex,
      headSlot,
    );

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

    // Not signed enough
    if (length * 3 > 1024) {
      sync_aggregate = syncAggregateResult.sync_aggregate;
      signature_slot = syncAggregateResult.slot;
      break;
    }

    // this is the next available slot after the nextBlockHeader slot which was not signed by enough validators
    nextHeaderSlotSearchIndex = syncAggregateResult.slot;
  }

  const {
    finalityHeader: prevFinalizedHeader,
    finalityHeaderBranch: prevFinalityBranch,
    syncCommittee,
    syncCommitteeBranch,
  } = await beaconApi.getPrevBlockHeaderStateInfo(from, signature_slot);

  const {
    finalityHeader: finalizedHeader,
    finalityHeaderBranch: finalityBranch,
  } = await beaconApi.getFinalityBlockAndProof(nextBlockHeader.slot);

  const {
    executionPayloadHeader: executionPayload,
    executionPayloadBranch: finalizedHeaderExecutionBranch,
  } = await beaconApi.getBlockExecutionPayloadAndProof(finalizedHeader.slot);

  return {
    proofInput: await getProofInput({
      prevBlockHeader,
      nextBlockHeader,
      prevFinalizedHeader,
      syncCommitteeBranch,
      syncCommittee,
      config: PRATER,
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
