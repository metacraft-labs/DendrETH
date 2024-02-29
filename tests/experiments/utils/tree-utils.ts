import { LevelIndexAndGIndex } from './common-utils';

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

export function* iterateTree(depth: bigint, lastLeafIndex?: bigint) {
  let indexOfLastNode = lastLeafIndex
    ? lastLeafIndex
    : fromDepth(depth).elementCount;
  let gIndexOfLastNode = lastLeafIndex
    ? indexToGIndex(lastLeafIndex, depth)
    : fromDepth(depth).levelEnd;

  for (let level = depth; level >= 1; level--) {
    for (let { indexOnThisLevel, gIndex } of iterateLevel(
      level,
      indexOfLastNode,
    )) {
      yield { indexOnThisLevel: indexOnThisLevel, gIndex, level };
    }
    gIndexOfLastNode = parentAndNeighbourFromGIndex(gIndexOfLastNode).parent;
    indexOfLastNode =
      level == 1n ? 1n : gIndexToIndex(gIndexOfLastNode, level - 1n);
  }
}

export function* iterateLevel(
  level: bigint,
  finalIndex?: bigint,
): Generator<{ indexOnThisLevel: bigint; gIndex: bigint }, void, unknown> {
  const { levelBeg, levelEnd } = fromDepth(level);
  const iterationBorder = finalIndex
    ? indexToGIndex(finalIndex, level)
    : levelEnd;
  for (let gIndex = levelBeg, idx = 0n; gIndex <= iterationBorder; gIndex++) {
    yield { indexOnThisLevel: ++idx, gIndex };
  }
}

export function* range(
  begin: bigint,
  end: bigint,
): Generator<{ indexOnThisLevel: bigint; gIndex: bigint }, void, unknown> {
  let indexOnThisLevel = 0n;
  for (let gIndex = begin; gIndex <= end; gIndex++) {
    yield { indexOnThisLevel: ++indexOnThisLevel, gIndex };
  }
}

export function childrenFromGIndex(gIndex: bigint): {
  leftChild: bigint;
  rightChild: bigint;
} {
  return {
    leftChild: gIndex * 2n,
    rightChild: gIndex * 2n + 1n,
  };
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

export function indexToGIndex(lastIndex: bigint, depth: bigint): bigint {
  return fromDepth(depth).levelBeg + lastIndex - 1n;
}

export function gIndexToIndex(gIndex: bigint, depth: bigint): bigint {
  return gIndex - fromDepth(depth).levelBeg + 1n;
}

export function gIndexToLevel(gIndex: bigint): bigint {
  return log2(gIndex);
}

export function isLeaf(gIndex: bigint, depth: bigint): boolean {
  return (
    gIndex >= fromDepth(depth).levelBeg && gIndex <= fromDepth(depth).levelEnd
  );
}

export function fromDepth(depth: bigint): {
  beg: bigint;
  end: bigint;
  levelBeg: bigint;
  levelEnd: bigint;
  elementCount: bigint;
} {
  return {
    beg: 1n,
    end: 2n ** depth - 1n,
    levelBeg: 2n ** (depth - 1n),
    levelEnd: 2n ** depth - 1n,
    elementCount: 2n ** (depth - 1n),
  };
}

export function log2(x: bigint) {
  let result = 0n;
  while (x > 1n) {
    x = x / 2n;
    result++;
  }
  return result;
}
