import { LevelIndexAndGIndex } from './common-utils';
import { Depth, GIndex, fromDepth, fromGIndex, log2 } from './gindex';

export interface TreeParams {
  depth: bigint;
  validatorCount: bigint;
  shouldExist: (gIndex: bigint) => boolean;
}

export type LeafData = {
  pubkey: Hash;
  withdrawal_credentials: Hash;
  effective_balance: Hash;
  activation_eligibility_epoch: Hash;
  activation_epoch: Hash;
  exit_epoch: Hash;
  withdrawable_epoch: Hash;
};

/// Hex string like 0x12abcd34
export type Hash = string;

export type NodeContent = {
  gIndex: bigint;
  hash: Hash;
  data?: LeafData;
};

export type NodeData = {
  gIndex: bigint;
  content: NodeContent;
  isLeaf?: boolean;
  isMissing?: boolean;
  isPlaceholder?: boolean;
};

export function* iterateTree({
  depth,
  lastLeafIndex,
}: {
  depth: bigint;
  lastLeafIndex?: bigint;
}) {
  console.log({ depth, lastLeafIndex });
  let lastIndex = lastLeafIndex ?? fromDepth(depth).levelEnd;

  // let lastGIndex = fromDepth(depth).levelStart + lastIndex;

  for (let level = depth; level >= 0; level--, lastIndex /= 2n) {
    console.log({ level, lastIndex });
    for (let { levelIndex, gIndex } of iterateLevel(level, lastIndex)) {
      // if (gIndex <= lastGIndex) {
      yield { levelIndex, gIndex, level };
      // }
    }
  }
}

export function min(a: bigint, b: bigint) {
  return a < b ? a : b;
}

export function* iterateLevel(
  depth: Depth,
  lastLeafNodeIndex?: bigint,
): Generator<LevelIndexAndGIndex> {
  const { levelStart, levelEnd } = fromDepth(depth);
  const end = lastLeafNodeIndex !== undefined ? levelStart + lastLeafNodeIndex : levelEnd;
  console.log({ levelStart, end });
  return yield* range(levelStart, min(levelEnd + 1n, end));
}

export function* range(
  begin: bigint,
  end: bigint,
): Generator<LevelIndexAndGIndex> {
  for (let i = begin; i < end; i++) {
    yield { gIndex: i, levelIndex: i - begin };
  }
}

export function parentAndNeighbourFromGIndex(gIndex: bigint): {
  parent: bigint;
  neighbour: bigint;
} {
  return gIndex % 2n == 0n
    ? {
        parent: gIndex / 2n,
        neighbour: gIndex + 1n,
      }
    : {
        parent: (gIndex - 1n) / 2n,
        neighbour: gIndex - 1n,
      };
}

export function indexToGIndex(lastIndex: GIndex, depth: Depth): bigint {
  return fromDepth(depth).levelStart + lastIndex - 1n;
}

export function gIndexToIndex(gIndex: GIndex, depth: Depth): bigint {
  return gIndex - fromDepth(depth).levelStart + 1n;
}

export function gIndexToLevel(gIndex: GIndex): Depth {
  return fromGIndex(gIndex).depth;
}

export function isLeaf(gIndex: GIndex, depth: Depth): boolean {
  return (
    gIndex >= fromDepth(depth).levelStart && gIndex <= fromDepth(depth).levelEnd
  );
}

// export function fromDepth(depth: bigint): {
//   beg: bigint;
//   end: bigint;
//   levelBeg: bigint;
//   levelEnd: bigint;
//   elementCount: bigint;
// } {
//   return {
//     beg: 1n,
//     end: 2n ** depth - 1n,
//     levelBeg: 2n ** (depth - 1n),
//     levelEnd: 2n ** depth - 1n,
//     elementCount: 2n ** (depth - 1n),
//   };
// }
