import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { Validator, ValidatorShaInput } from '@dendreth/relay/types/types';
import { bitsToHex } from '@dendreth/utils/ts-utils/hex-utils';

export function gindexFromIndex(index: bigint, depth: bigint): bigint {
  return 2n ** depth + index;
}

export function indexFromGindex(gindex: bigint, depth: bigint): bigint {
  return gindex - 2n ** depth;
}

export function getDepthByGindex(gindex: number): number {
  return Math.floor(Math.log2(gindex));
}

export function getNthParent(gindex: bigint, n: bigint): bigint {
  return gindex / 2n ** n;
}

export function getParent(gindex: bigint): bigint {
  return getNthParent(gindex, 1n);
}

export function getLastSlotInEpoch(epoch: bigint): bigint {
  return epoch * 32n + 31n;
}

export function getFirstSlotInEpoch(epoch: bigint): bigint {
  return epoch * 32n;
}

// TODO: make indices be a number[]
export function* makeBranchIterator(indices: bigint[], depth: bigint) {
  const changedValidatorGindices = indices.map(index =>
    gindexFromIndex(index, depth),
  );

  yield changedValidatorGindices;

  let nodesNeedingUpdate = new Set(changedValidatorGindices.map(getParent));
  while (nodesNeedingUpdate.size !== 0) {
    const newNodesNeedingUpdate = new Set<bigint>();

    for (const gindex of nodesNeedingUpdate) {
      if (gindex !== 1n) {
        newNodesNeedingUpdate.add(getParent(gindex));
      }
    }

    yield [...nodesNeedingUpdate];
    nodesNeedingUpdate = newNodesNeedingUpdate;
  }
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

  while (gindex !== 1n) {
    const siblingGindex = gindex % 2n === 0n ? gindex + 1n : gindex - 1n;

    const hash = await redis.extractHashFromCommitmentMapperProof(
      siblingGindex,
      epoch,
      hashAlg,
    );
    if (hash !== null) {
      path.push(hash as any);
    }

    gindex = gindex / 2n;
  }

  if (hashAlg === 'sha256') {
    path = (path as any).map(bitsToHex);
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
