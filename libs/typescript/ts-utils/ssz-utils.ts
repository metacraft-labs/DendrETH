import { Type } from '@chainsafe/ssz';
import { formatHex, hexToBytes } from './bls';
import { readFile } from 'fs/promises';
import { sha256 } from 'ethers/lib/utils';

export function verifyMerkleProof(
  branch: string[],
  hashTreeRoot: string,
  leaf: string,
  index: bigint,
): boolean {
  let hash = leaf;

  for (const proofElement of branch) {
    hash = sha256(
      '0x' +
        (index % 2n === 0n ? [hash, proofElement] : [proofElement, hash])
          .map(formatHex)
          .join(''),
    );

    index /= 2n;
  }

  return formatHex(hash) === formatHex(hashTreeRoot);
}

export function hashTreeRoot(_leaves: string[]) {
  const leaves = [..._leaves];

  const UPPER_SIZE = leaves.length;
  let n = 2;

  while (UPPER_SIZE / n >= 1) {
    for (let i = 0; i < UPPER_SIZE / n; i++) {
      leaves[i] = sha256(
        '0x' + formatHex(leaves[2 * i]) + formatHex(leaves[2 * i + 1]),
      );
    }

    n *= 2;
  }

  return leaves[0];
}

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
