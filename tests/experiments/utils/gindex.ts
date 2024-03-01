/*
 * Conventions
 * GIndex: 1-based index representing a node position in a binary tree
 * Depth: 0-based index representing the level of a node in a binary tree
 * LevelStart: GIndex of the first node in a level
 * LevelIndex: 0-based index representing a node position in a level
 */

export type GIndex = bigint;
export type Depth = bigint;

export function fromGIndex(gIndex: GIndex): GIndexWrapper {
  return new GIndexWrapper(gIndex);
}

export class GIndexWrapper {
  constructor(public readonly gIndex: bigint) {}

  get depth(): Depth {
    return log2(this.gIndex);
  }

  get levelStart(): GIndex {
    return 2n ** log2(this.gIndex);
  }

  get levelIndex(): bigint {
    return this.gIndex - fromDepth(this.depth).levelStart + 1n;
  }

  get left(): GIndex {
    return 2n * this.gIndex;
  }

  get right(): GIndex {
    return 2n * this.gIndex + 1n;
  }

  get parent(): GIndex {
    return this.gIndex / 2n;
  }
}

export function fromDepth(depth: bigint): DepthWrapper {
  return new DepthWrapper(depth);
}

export class DepthWrapper {
  public readonly depth: bigint;
  constructor(depth: bigint | number) {
    this.depth = BigInt(depth);
  }

  get first(): GIndex {
    return 1n;
  }

  get last(): GIndex {
    return this.levelEnd;
  }

  get levelStart(): GIndex {
    return 2n ** this.depth;
  }

  get levelEnd(): GIndex {
    return 2n ** (this.depth + 1n) - 1n;
  }

  get elementCount(): bigint {
    return this.last;
  }
}

export function log2(x: bigint) {
  let result = 0n;
  while (x > 1n) {
    x = x / 2n;
    result++;
  }
  return result;
}
