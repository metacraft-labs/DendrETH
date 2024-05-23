import { sha256 } from 'ethers';
import { IBeaconApi } from '@dendreth/relay/abstraction/beacon-api-interface';
import { Config } from '@dendreth/relay/constants/constants';

export const getConstructorArgs = async (
  beaconApi: IBeaconApi,
  slot: number,
  config: Config,
) => {
  const { ssz } = await import('@lodestar/types');

  const finalizedBlockHeader = await beaconApi.getFinalizedBlockHeader(slot);
  const finalizedHeaderRoot =
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(finalizedBlockHeader);

  const executioStateRoot = await beaconApi.getExecutionStateRoot(slot);

  let result = sha256(
    config.FORK_VERSION.padEnd(66, '0') +
      config.GENESIS_VALIDATORS_ROOT.slice(2),
  );

  return [
    finalizedHeaderRoot,
    finalizedBlockHeader.slot,
    finalizedHeaderRoot,
    executioStateRoot,
    config.DOMAIN_SYNC_COMMITTEE + result.slice(2, 58),
  ];
};
