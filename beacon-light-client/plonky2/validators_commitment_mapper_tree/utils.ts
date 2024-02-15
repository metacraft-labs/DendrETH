import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';

export function gindexFromIndex(index: bigint, depth: bigint) {
  return 2n ** depth - 1n + index;
}

export function getNthParent(gindex: bigint, n: bigint) {
  return (gindex - 2n ** n + 1n) / 2n ** n;
}

export function getParent(gindex: bigint) {
  return getNthParent(gindex, 1n);
}

export function* makeBranchIterator(indices: bigint[], depth: bigint) {
  const changedValidatorGindices = indices.map(index =>
    gindexFromIndex(index, depth),
  );

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

type HashAlgorithm = 'sha256' | 'poseidon';

function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += 2 ** (7 - bit) * hash[byte * 8 + bit];
    }
    result[byte] = value;
  }
  return result;
}

type PoseidonOrSha256<T extends 'sha256' | 'poseidon'> = T extends 'sha256'
  ? string[]
  : number[][];

export async function getCommitmentMapperProof<T extends 'sha256' | 'poseidon'>(
  epoch: bigint,
  gindex: bigint,
  hashAlg: T,
  redis: RedisLocal,
): Promise<PoseidonOrSha256<T>> {
  let path: PoseidonOrSha256<T> = [];

  while (gindex !== 0n) {
    const siblingGindex = gindex % 2n === 0n ? gindex - 1n : gindex + 1n;

    const hash = await redis.extractHashFromCommitmentMapperProof(
      siblingGindex,
      epoch,
      hashAlg,
    );
    if (hash !== null) {
      path.push(hash as any);
    }

    gindex = (gindex - 1n) / 2n;
  }

  if (hashAlg === 'sha256') {
    path = (path as any).map(bitArrayToByteArray).map(bytesToHex);
  }

  return path;
}
