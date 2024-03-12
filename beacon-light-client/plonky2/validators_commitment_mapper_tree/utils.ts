import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { Validator, ValidatorShaInput } from '../../../relay/types/types';

export function gindexFromIndex(index: bigint, depth: bigint) {
  return 2n ** depth - 1n + index;
}

export function getDepthByGindex(gindex: number): number {
  return Math.floor(Math.log2(gindex + 1));
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
  : string[][];

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

export function convertValidatorToProof(
  validator: Validator,
  ssz: any,
): ValidatorShaInput {
  return {
    pubkey: bytesToHex(validator.pubkey),
    withdrawalCredentials: bytesToHex(validator.withdrawalCredentials),
    effectiveBalance: bytesToHex(
      ssz.phase0.Validator.fields.effectiveBalance.hashTreeRoot(
        validator.effectiveBalance,
      ),
    ),
    slashed: bytesToHex(
      ssz.phase0.Validator.fields.slashed.hashTreeRoot(validator.slashed),
    ),
    activationEligibilityEpoch: bytesToHex(
      ssz.phase0.Validator.fields.activationEligibilityEpoch.hashTreeRoot(
        validator.activationEligibilityEpoch,
      ),
    ),
    activationEpoch: bytesToHex(
      ssz.phase0.Validator.fields.activationEpoch.hashTreeRoot(
        validator.activationEpoch,
      ),
    ),
    exitEpoch: bytesToHex(
      ssz.phase0.Validator.fields.exitEpoch.hashTreeRoot(validator.exitEpoch),
    ),
    withdrawableEpoch: bytesToHex(
      ssz.phase0.Validator.fields.withdrawableEpoch.hashTreeRoot(
        validator.withdrawableEpoch,
      ),
    ),
  };
}

export function getZeroValidatorInput() {
  return {
    pubkey: ''.padEnd(96, '0'),
    withdrawalCredentials: ''.padEnd(64, '0'),
    effectiveBalance: ''.padEnd(64, '0'),
    slashed: ''.padEnd(64, '0'),
    activationEligibilityEpoch: ''.padEnd(64, '0'),
    activationEpoch: ''.padEnd(64, '0'),
    exitEpoch: ''.padEnd(64, '0'),
    withdrawableEpoch: ''.padEnd(64, '0'),
  };
}
