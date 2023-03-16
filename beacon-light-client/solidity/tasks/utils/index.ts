import { checkConfig } from '../../../../libs/typescript/ts-utils/common-utils';
import { getBlockHeaderFromUpdate } from '../../../../libs/typescript/ts-utils/ssz-utils';

// TODO: should get the finalized header for the slot
export const getConstructorArgs = async (slot: number) => {
  const config = {
    BEACON_REST_API: process.env.BEACON_REST_API,
  };

  checkConfig(config);

  const blockHeader = await (
    await fetch(`${config.BEACON_REST_API}/eth/v1/beacon/headers/` + slot)
  ).json();

  const finality_checkpoints = await (
    await fetch(
      `${config.BEACON_REST_API}/eth/v1/beacon/states/` +
        slot +
        `/finality_checkpoints`,
    )
  ).json();
  const block = await (
    await fetch(`${config.BEACON_REST_API}/eth/v2/beacon/blocks/` + slot)
  ).json();

  const { ssz } = await import('@lodestar/types');

  return [
    ssz.phase0.BeaconBlockHeader.hashTreeRoot(
      await getBlockHeaderFromUpdate(blockHeader.data.header.message),
    ),
    finality_checkpoints.data.finalized.root,
    block.data.message.body.execution_payload.state_root,
  ];
};
