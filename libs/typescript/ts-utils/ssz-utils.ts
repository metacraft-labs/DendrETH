import { Type } from '@chainsafe/ssz';
import { hexToBytes } from './bls';
import { readFile } from 'fs/promises';

export async function jsonToSerializedBase64<T>(
  sszType: Type<T>,
  path: string,
) {
  const jsonContent = JSON.parse(await readFile(path, 'utf-8'));
  const data = sszType.fromJson(jsonContent);
  const serializedData = sszType.serialize(data);
  var b64Data = Buffer.from(serializedData).toString('base64');
  return b64Data;
}

export async function getBlockHeaderFromUpdate(head) {
  const { ssz } = await import('@lodestar/types');

  const blockHeader = ssz.phase0.BeaconBlockHeader.defaultValue();
  blockHeader.slot = Number.parseInt(head.slot);
  blockHeader.proposerIndex = Number.parseInt(head.proposer_index);
  blockHeader.parentRoot = hexToBytes(head.parent_root);
  blockHeader.stateRoot = hexToBytes(head.state_root);
  blockHeader.bodyRoot = hexToBytes(head.body_root);

  return blockHeader;
}

const EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
const SLOTS_PER_EPOCH = 32;

export function computeSyncCommitteePeriodAt(slot: number) {
  return Math.floor(
    slot / (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH),
  );
}
