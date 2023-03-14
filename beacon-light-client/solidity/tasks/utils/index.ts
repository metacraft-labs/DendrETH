import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';
import { getBlockHeaderFromUpdate } from '../../../../libs/typescript/ts-utils/ssz-utils';

export const getConstructorArgs = async (apiUrl: string, slot: number) => {
  const blockHeader = await (
    await fetch(`${apiUrl}/eth/v1/beacon/headers/` + slot)
  ).json();
  const finality_checkpoints = await (
    await fetch(
      `${apiUrl}/eth/v1/beacon/states/` + slot + `/finality_checkpoints`,
    )
  ).json();
  const block = await (
    await fetch(`${apiUrl}/eth/v2/beacon/blocks/` + slot)
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
