import { checkConfig } from '../../../../libs/typescript/ts-utils/common-utils';
import { IBeaconApi } from '../../../../relay/abstraction/beacon-api-interface';

// TODO: should get the finalized header for the slot
export const getConstructorArgs = async (
  beaconApi: IBeaconApi,
  slot: number,
) => {
  const config = {
    BEACON_REST_API: process.env.BEACON_REST_API,
  };

  checkConfig(config);

  const { ssz } = await import('@lodestar/types');

  const finalizedBlockHeader = await beaconApi.getFinalizedBlockHeader(slot);
  const finalizedHeaderRoot =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedBlockHeader);

  const executioStateRoot = await beaconApi.getExecutionStateRoot(slot);

  return [finalizedHeaderRoot, finalizedHeaderRoot, executioStateRoot];
};
