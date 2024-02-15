
export function gindexFromValidatorIndex(index: bigint, depth: bigint) {
  return (2n ** depth) - 1n + index;
}

export function getParent(gindex: bigint) {
  return (gindex - 1n) / 2n;
}

export function* makeBranchIterator(indices: bigint[], depth: bigint) {
  const changedValidatorGindices = indices.map(index => gindexFromValidatorIndex(index, depth));

  yield changedValidatorGindices;

  let nodesNeedingUpdate = new Set(changedValidatorGindices.map(getParent));
  while (nodesNeedingUpdate.size !== 0) {
    const newNodesNeedingUpdate = new Set<bigint>();

    for (const gindex of nodesNeedingUpdate) {
      if (gindex !== 0n) {
        newNodesNeedingUpdate.add(getParent(gindex));
      }
    }

    yield [...nodesNeedingUpdate];
    nodesNeedingUpdate = newNodesNeedingUpdate;
  }
}
