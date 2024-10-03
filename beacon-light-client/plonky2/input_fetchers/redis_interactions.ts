import { Redis } from '@dendreth/relay/implementations/redis';
import JSONbig from 'json-bigint';
import CONSTANTS from '/home/xearty/code/repos/DendrETH/beacon-light-client/plonky2/kv_db_constants.json';
import { getDepthByGindex } from '@dendreth/utils/ts-utils/common-utils';
import {
  BalanceProof,
  BalancesAccumulatorInput,
  BlsDepositData,
} from '@dendreth/relay/types/types';

export async function getSlotWithLatestChange(
  redis: Redis,
  key: string,
  slot: bigint,
): Promise<bigint | null> {
  const values = await redis.client.zrange(
    `${key}:${CONSTANTS.slotLookupKey}`,
    slot.toString(),
    0,
    'BYSCORE',
    'REV',
    'LIMIT',
    0,
    1,
  );
  if (values.length === 0) {
    return null;
  }
  return BigInt(values[0]);
}

export async function collectOutdatedSlots(
  redis: Redis,
  key: string,
  newOldestSlot: bigint,
): Promise<bigint[]> {
  const slotWithLatestChange = await getSlotWithLatestChange(
    redis,
    key,
    newOldestSlot,
  );
  if (slotWithLatestChange !== null) {
    return (
      await redis.client.zrange(
        `${key}:${CONSTANTS.slotLookupKey}`,
        0,
        (slotWithLatestChange - 1n).toString(),
        'BYSCORE',
      )
    ).map(BigInt);
  }
  return [];
}

export async function extractHashFromCommitmentMapperProof(
  redis: Redis,
  gindex: bigint,
  slot: bigint,
  hashAlgorithm: 'sha256' | 'poseidon',
): Promise<number[] | null> {
  const hashAlgorithmOptionMap = {
    sha256: 'sha256HashTreeRoot',
    poseidon: 'poseidonHashTreeRoot',
  };

  const hashKey = hashAlgorithmOptionMap[hashAlgorithm];

  const latestSlot = await getSlotWithLatestChange(
    redis,
    `${CONSTANTS.validatorProofKey}:${gindex}`,
    slot,
  );
  if (latestSlot === null) {
    const depth = getDepthByGindex(Number(gindex));
    const result = await redis.client.get(
      `${CONSTANTS.validatorProofKey}:zeroes:${depth}`,
    );

    if (result == null) {
      return null;
    }

    return JSONbig.parse(result).publicInputs[hashKey];
  }

  const key = `${CONSTANTS.validatorProofKey}:${gindex}:${latestSlot}`;
  const result = await redis.client.get(key);
  if (result == null) {
    return null;
  }

  return JSONbig.parse(result).publicInputs[hashKey];
}

export async function getValidatorsCommitmentRoot(
  redis: Redis,
  slot: bigint,
): Promise<string[] | null> {
  const prefix = `${CONSTANTS.validatorProofKey}:1`;
  const latestRootChangeSlot = await getSlotWithLatestChange(
    redis,
    prefix,
    slot,
  );

  if (latestRootChangeSlot == null) return null;

  const rootData = await redis.client.get(`${prefix}:${latestRootChangeSlot}`);
  if (rootData == null) return null;

  const obj = JSON.parse(rootData);
  const poseidonHash = obj.poseidonHash;
  return poseidonHash;
}

export async function getValidatorsRoot(
  redis: Redis,
  slot: bigint,
): Promise<String | null> {
  return redis.client.get(`${CONSTANTS.validatorsRootKey}:${slot}`);
}

export async function isZeroBalanceEmpty(redis: Redis) {
  const result = await redis.client.get(
    `${CONSTANTS.validatorBalanceInputKey}:${CONSTANTS.validatorRegistryLimit}`,
  );

  return result == null;
}

export async function saveBalancesAccumulatorProof(
  redis: Redis,
  protocol: string,
  level: bigint,
  index: bigint,
  proof: BalanceProof = {
    needsChange: true,
    proofKey: '',
    publicInputs: {
      rangeTotalValue: '0',
      rangeValidatorCommitment: [0, 0, 0, 0],
      rangeBalancesRoot: ''.padEnd(64, '0'),
      withdrawalCredentials: [''.padEnd(64, '0')],
      currentEpoch: '0',
      numberOfNonActivatedValidators: 0,
      numberOfActiveValidators: 0,
      numberOfExitedValidators: 0,
      numberOfSlashedValidators: 0,
    },
  },
): Promise<void> {
  await redis.client.set(
    `${CONSTANTS.balanceVerificationAccumulatorProofKey}:${protocol}:${level}:${index}`,
    JSON.stringify(proof),
  );
}

export async function saveBalancesAccumulatorInput(
  redis: Redis,
  balancesInputs: BalancesAccumulatorInput[],
  protocol: string,
) {
  const args = balancesInputs
    .map((input, index) => {
      return [
        `${CONSTANTS.balanceVerificationAccumulatorKey}:${protocol}:${index}`,
        JSON.stringify(input),
      ];
    })
    .flat();

  await redis.client.mset(...args);
}

export async function saveValidatorBalancesInput(
  redis: Redis,
  protocol: string,
  inputsWithIndices: { index: number; input: any }[],
) {
  const args = inputsWithIndices
    .map(ii => {
      return [
        `${protocol}:${CONSTANTS.validatorBalanceInputKey}:${ii.index}`,
        JSON.stringify(ii.input),
      ];
    })
    .flat();

  await redis.client.mset(...args);
}

export async function saveFinalProofInput(
  redis: Redis,
  protocol: string,
  input: {
    stateRoot: string;
    stateRootBranch: string[];
    blockRoot: string;
    slot: string;
    slotBranch: string[];
    balancesBranch: string[];
    validatorsBranch: string[];
  },
) {
  await redis.client.set(
    `${protocol}:${CONSTANTS.finalProofInputKey}`,
    JSON.stringify(input),
  );
}

export async function saveBalanceAggregatorFinalProofInput(
  redis: Redis,
  protocol: string,
  input: {
    blockRoot: string;
    stateRoot: string;
    stateRootBranch: string[];
    validatorsBranch: string[];
    balanceBranch: string[];
    executionBlockNumber: string;
    executionBlockNumberBranch: string[];
    slot: string;
    slotBranch: string[];
  },
) {
  await redis.client.set(
    `${protocol}:${CONSTANTS.depositBalanceVerificationFinalProofInputKey}`,
    JSON.stringify(input),
  );
}

export async function saveBalanceProof(
  redis: Redis,
  protocol: string,
  level: bigint,
  index: bigint,
  proof: BalanceProof = {
    needsChange: true,
    proofKey: '',
    publicInputs: {
      rangeTotalValue: '0',
      rangeValidatorCommitment: [0, 0, 0, 0],
      rangeBalancesRoot: ''.padEnd(64, '0'),
      withdrawalCredentials: [''.padEnd(64, '0')],
      currentEpoch: '0',
      numberOfNonActivatedValidators: 0,
      numberOfActiveValidators: 0,
      numberOfExitedValidators: 0,
      numberOfSlashedValidators: 0,
    },
  },
): Promise<void> {
  await redis.client.set(
    `${protocol}:${CONSTANTS.balanceVerificationProofKey}:${level}:${index}`,
    JSON.stringify(proof),
  );
}

export async function getDepositsCount(redis: Redis): Promise<number> {
  const pattern = `${CONSTANTS.depositSignatureVerificationKey}:*`;
  const keys = await redis.client.keys(pattern);

  return keys.length;
}

export async function saveDeposit(
  redis: Redis,
  index: number,
  deposit: BlsDepositData,
): Promise<void> {
  await redis.client.set(
    `${CONSTANTS.depositSignatureVerificationKey}:${index}`,
    JSON.stringify(deposit),
  );
}

export async function extractHashFromDepositCommitmentMapperProof(
  redis: Redis,
  protocol: string,
  gindex: bigint,
  hashAlgorithm: 'sha256' | 'poseidon',
): Promise<number[] | null> {
  const hashAlgorithmOptionMap = {
    sha256: 'sha256HashTreeRoot',
    poseidon: 'poseidonHashTreeRoot',
  };

  const hashKey = hashAlgorithmOptionMap[hashAlgorithm];

  const result = await redis.client.get(
    `${protocol}:${CONSTANTS.balanceVerificationAccumulatorProofKey}:${gindex}`,
  );
  if (result === null) {
    const depth = getDepthByGindex(Number(gindex));
    const result = await redis.client.get(
      `${protocol}:${CONSTANTS.balanceVerificationAccumulatorProofKey}:zeroes:${depth}`,
    );

    if (result == null) {
      return null;
    }

    return JSONbig.parse(result).publicInputs[hashKey];
  }

  return JSONbig.parse(result).publicInputs[hashKey];
}

export async function saveDepositBalanceVerificationInput(
  redis: Redis,
  protocol: string,
  index: bigint,
  input: any,
): Promise<void> {
  await redis.client.set(
    `${protocol}:${CONSTANTS.depositBalanceVerificationInputKey}:${index}`,
    JSONbig.stringify(input),
  );
}

export async function saveDepositBalanceVerificationProof(
  redis: Redis,
  protocol: string,
  level: bigint,
  index: bigint,
): Promise<void> {
  const obj = {
    needsChange: true,
    proofKey: '',
    publicInputs: {
      currentEpoch: '0',
      validatorsCommitmentMapperRoot: [0, 0, 0, 0],
      balancesRoot: ''.padEnd(64, '0'),
      pubkeyCommitmentMapperRoot: [0, 0, 0, 0],
      accumulatedData: {
        balance: '0',
        validatorStatusStats: {
          nonActivatedCount: 0,
          activeCount: 0,
          exitedCount: 0,
          slashedCount: 0,
        },
      },
    },
  };

  await redis.client.set(
    `${protocol}:${CONSTANTS.depositBalanceVerificationProofKey}:${level}:${index}`,
    JSONbig.stringify(obj),
  );
}

export async function getBalanceWrapperProofWithPublicInputs(
  redis: Redis,
  protocol: string,
): Promise<any> {
  return redis.client.get(
    `${protocol}:balance_wrapper_proof_with_public_inputs`,
  );
}

export async function getBalanceWrapperVerifierOnly(
  redis: Redis,
  protocol: string,
): Promise<any> {
  return redis.client.get(`${protocol}:balance_wrapper_verifier_only`);
}
