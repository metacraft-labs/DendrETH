import { readFile } from 'fs/promises';

import { sha256 } from 'ethers/lib/utils';

import { Type } from '@chainsafe/ssz';

import { formatHex, hexToBytes } from './bls';

import type { ssz } from '@lodestar/types';

export type SSZ = typeof ssz;
export type Phase0 = typeof ssz.phase0;
export type Deneb = typeof ssz.deneb;
export type Capella = typeof ssz.capella;
export type CapellaOrDeneb = Capella & Deneb;

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

function nearestUpperPowerOfTwo(num) {
  let power = Math.ceil(Math.log2(num));
  return Math.pow(2, power);
}

export function hashTreeRoot(_leaves: string[], treeDepth: number): string {
  const zero_hashes: string[] = [];

  zero_hashes[0] = '0x' + '0'.repeat(64);

  for (let height = 0; height < treeDepth - 1; height++) {
    zero_hashes[height + 1] = sha256(
      '0x' + formatHex(zero_hashes[height]) + formatHex(zero_hashes[height]),
    );
  }

  const leavesLength = _leaves.length;
  const treeLeaves = nearestUpperPowerOfTwo(leavesLength);
  const leaves = [
    ..._leaves,
    ...Array(treeLeaves - leavesLength).fill('0x' + '0'.repeat(64)),
  ];

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

  let subtreeRoot = leaves[0];

  for (let i = Math.log2(treeLeaves); i < treeDepth; i++) {
    subtreeRoot = sha256(
      '0x' + formatHex(subtreeRoot) + formatHex(zero_hashes[i]),
    );
  }

  return subtreeRoot;
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

export const SLOTS_PER_PERIOD = 8192;

export function computeSyncCommitteePeriodAt(slot: number) {
  return Math.floor(slot / SLOTS_PER_PERIOD);
}

export function computeEpochAt(slot: number) {
  return Math.floor(slot / 32);
}
