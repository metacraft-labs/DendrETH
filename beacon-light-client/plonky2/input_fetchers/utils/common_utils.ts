import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { CommitmentMapperInput, Validator } from '@dendreth/relay/types/types';
import { bitsToHex } from '@dendreth/utils/ts-utils/hex-utils';

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

export async function getDepositCommitmentMapperProof<
  T extends 'sha256' | 'poseidon',
>(
  protocol: string,
  gindex: bigint,
  hashAlg: T,
  redis: RedisLocal,
): Promise<PoseidonOrSha256<T>> {
  let path: PoseidonOrSha256<T> = [];

  while (gindex !== 1n) {
    const siblingGindex = gindex % 2n === 0n ? gindex + 1n : gindex - 1n;

    const hash = await redis.extractHashFromDepositCommitmentMapperProof(
      protocol,
      siblingGindex,
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

function numberToString(num: number): string {
  return num == Infinity ? BigInt(2n ** 64n - 1n).toString() : num.toString();
}

export function commitmentMapperInputFromValidator(
  validator: Validator,
): CommitmentMapperInput {
  return {
    validator: {
      pubkey: bytesToHex(validator.pubkey),
      withdrawalCredentials: bytesToHex(validator.withdrawalCredentials),
      effectiveBalance: numberToString(validator.effectiveBalance),
      slashed: validator.slashed,
      activationEligibilityEpoch: numberToString(
        validator.activationEligibilityEpoch,
      ),
      activationEpoch: numberToString(validator.activationEpoch),
      exitEpoch: numberToString(validator.exitEpoch),
      withdrawableEpoch: numberToString(validator.withdrawableEpoch),
    },
    isReal: true,
  };
}

export function createDummyCommitmentMapperInput(): CommitmentMapperInput {
  return {
    validator: {
      pubkey: ''.padEnd(96, '0'),
      withdrawalCredentials: ''.padEnd(64, '0'),
      effectiveBalance: '0',
      slashed: false,
      activationEligibilityEpoch: '0',
      activationEpoch: '0',
      exitEpoch: '0',
      withdrawableEpoch: '0',
    },
    isReal: false,
  };
}
