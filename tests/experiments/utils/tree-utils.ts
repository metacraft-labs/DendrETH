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

export function* iterateLevel(
  level: bigint,
): Generator<{ indexOnThisLevel: bigint; gIndex: bigint }, void, unknown> {
  const { levelBeg, levelEnd } = fromDepth(level);
  for (let gIndex = levelBeg, idx = 0n; gIndex <= levelEnd; gIndex++) {
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

export function fromGIndex(gIndex: bigint): {
  leftChild: bigint;
  rightChild: bigint;
} {
  return {
    leftChild: gIndex * 2n,
    rightChild: gIndex * 2n + 1n,
  };
}

export function isLeaf(gIndex: bigint, depth: bigint): boolean {
  return gIndex >= fromDepth(depth).levelBeg;
}

export function fromDepth(depth: bigint): {
  beg: bigint;
  end: bigint;
  levelBeg: bigint;
  levelEnd: bigint;
} {
  return {
    beg: 1n,
    end: 2n ** depth - 1n,
    levelBeg: 2n ** (depth - 1n),
    levelEnd: 2n ** depth - 1n,
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
