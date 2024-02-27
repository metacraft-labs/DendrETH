import { LevelIndexAndGIndex } from './common-utils';

export interface TreeParams {
  depth: bigint;
  lastValidatorIndex: bigint;
  shouldExist: (gIndex: bigint) => boolean;
}

export type LeafData = {
  pubkey: string;
  withdrawal_credentials: string;
  effective_balance: string;
  activation_eligibility_epoch: string;
  activation_epoch: string;
  exit_epoch: string;
  withdrawable_epoch: string;
};

export type NodeContent = {
  gIndex: bigint;
  hash: string;
  data?: LeafData | string;
};

export type NodeData = {
  gIndex: bigint;
  content: NodeContent;
  isLeaf?: boolean;
  isMissing?: boolean;
  isPlaceholder?: boolean;
};

export function iterateLevel(
  level: bigint,
): Generator<{ indexOnThisLevel: bigint; gIndex: bigint }, void, unknown> {
  const { levelBeg, levelEnd } = fromDepth(level);
  return range(levelBeg, levelEnd);
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

export function fromGI(gIndex: bigint): {
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
