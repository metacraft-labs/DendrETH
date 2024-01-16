import {
  splitIntoBatches,
} from '../../libs/typescript/ts-utils/common-utils';
import { hexToBytes } from '../../libs/typescript/ts-utils/bls';
import { bitsToBytes } from '../../libs/typescript/ts-utils/hex-utils';
import { IRedis } from '../abstraction/redis-interface';
import {
  BalanceProof,
  ProofResultType,
  Validator,
  ValidatorProof,
} from '../types/types';
import { createClient, RedisClientType } from 'redis';
import validator_commitment_constants from '../../beacon-light-client/plonky2/constants/validator_commitment_constants.json';

export class Redis implements IRedis {
  private redisClient: RedisClientType;
  private pubSub: RedisClientType;

  constructor(redisHost: string, redisPort: number) {
    this.redisClient = createClient({
      url: `redis://${redisHost}:${redisPort}`,
    });

    this.pubSub = this.redisClient.duplicate();
  }

  async disconnect() {
    await this.waitForConnection();
    await this.pubSub.disconnect();
    await this.redisClient.disconnect();
  }

  async addToEpochLookup(key: string, epoch: bigint) {
    await this.waitForConnection();

    await this.redisClient.zAdd(`${key}:${validator_commitment_constants.epochLookupKey}`, { score: Number(epoch), value: epoch.toString() });
  }

  async getLatestEpoch(key: string, epoch: bigint): Promise<bigint | null> {
    await this.waitForConnection();

    const values = await this.redisClient.zRange(`${key}:${validator_commitment_constants.epochLookupKey}`, epoch.toString(), 0, { BY: 'SCORE', REV: true, LIMIT: { offset: 0, count: 1 } });
    if (values.length === 0) {
      return null;
    }
    return BigInt(values[0]);
  }

  async pruneOldEpochs(key: string, newOldestEpoch: bigint): Promise<number> {
    await this.waitForConnection();

    const latestEpoch = await this.getLatestEpoch(key, newOldestEpoch);
    if (latestEpoch !== null) {
      const range = await this.redisClient.zRange(`${key}:${validator_commitment_constants.epochLookupKey}`, 0, (latestEpoch - 1n).toString(), { BY: 'SCORE' });
      if (range.length !== 0) {
        await this.redisClient.zRem(`${key}:${validator_commitment_constants.epochLookupKey}`, range);
        return await this.redisClient.del(range.map((suffix) => `${key}:${suffix}`));
      }
    }
    return 0;
  }

  async getAllKeys(pattern: string): Promise<string[]> {
    await this.waitForConnection();
    return this.redisClient.keys(pattern);
  }

  async extractHashFromCommitmentMapperProof(gindex: bigint, epoch: bigint, hashAlgorithm: 'sha256' | 'poseidon'): Promise<number[] | null> {
    const hashAlgorithmOptionMap = {
      sha256: 'sha256Hash',
      poseidon: 'poseidonHash',
    };

    const hashKey = hashAlgorithmOptionMap[hashAlgorithm];

    const latestEpoch = await this.getLatestEpoch(`${validator_commitment_constants.validatorProofKey}:${gindex}`, BigInt(epoch));
    if (latestEpoch === null) {
      const depth = Math.floor(Math.log2(Number(gindex) + 1));
      const result = await this.redisClient.json.get(`${validator_commitment_constants.validatorProofKey}:zeroes:${depth}`, { path: hashKey }) as any;
      return result;
    }

    const key = `${validator_commitment_constants.validatorProofKey}:${gindex}:${latestEpoch}`;
    return this.redisClient.json.get(key, { path: hashKey }) as any;
  }

  async notifyAboutNewProof(): Promise<void> {
    await this.waitForConnection();

    this.pubSub.publish('proofs_channel', 'proof');
  }

  async getValidatorsBatched(ssz, batchSize = 1000): Promise<Validator[]> {
    await this.waitForConnection();

    const keys = (await this.redisClient.keys(
      `${validator_commitment_constants.validatorKey}:*`,
    )).filter((key) => !key.includes(validator_commitment_constants.epochLookupKey));

    if (keys.length === 0) {
      return [];
    }

    let allValidators: Validator[] = new Array(keys.length);

    for (const [keyBatchIndex, batchKeys] of splitIntoBatches(keys, batchSize).entries()) {
      const batchValidators = (await this.redisClient.json.mGet(batchKeys, '$')).map((entry) => entry![0]);
      console.log(batchValidators);

      for (const [index, redisValidator] of batchValidators.entries()) {
        try {
          const validator: Validator = {
            pubkey: hexToBytes(redisValidator.pubkey),
            withdrawalCredentials: hexToBytes(
              redisValidator.withdrawalCredentials,
            ),
            effectiveBalance:
              ssz.phase0.Validator.fields.effectiveBalance.deserialize(
                hexToBytes(redisValidator.effectiveBalance).slice(0, 8),
              ),

            slashed: ssz.phase0.Validator.fields.slashed.deserialize(
              hexToBytes(redisValidator.slashed).slice(0, 1),
            ),
            activationEligibilityEpoch:
              ssz.phase0.Validator.fields.activationEligibilityEpoch.deserialize(
                hexToBytes(redisValidator.activationEligibilityEpoch).slice(
                  0,
                  8,
                ),
              ),
            activationEpoch:
              ssz.phase0.Validator.fields.activationEpoch.deserialize(
                hexToBytes(redisValidator.activationEpoch).slice(0, 8),
              ),
            exitEpoch: ssz.phase0.Validator.fields.exitEpoch.deserialize(
              hexToBytes(redisValidator.exitEpoch).slice(0, 8),
            ),
            withdrawableEpoch:
              ssz.phase0.Validator.fields.withdrawableEpoch.deserialize(
                hexToBytes(redisValidator.withdrawableEpoch).slice(0, 8),
              ),
          };

          const validatorIndex = Number(batchKeys[index].split(':')[1]);
          allValidators[validatorIndex] = validator;
        } catch (e) {
          console.log(e);
          continue;
        }

      }
      console.log(`Loaded batch, ${keyBatchIndex / batchSize}/${keys.length / batchSize}`);
    }

    return allValidators;
  }

  async isZeroValidatorEmpty() {
    await this.waitForConnection();

    const result = await this.redisClient.json.get(
      `${validator_commitment_constants.validatorKey}:${validator_commitment_constants.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async isZeroBalanceEmpty() {
    await this.waitForConnection();

    const result = await this.redisClient.json.get(
      `${validator_commitment_constants.validatorBalanceInputKey}:${validator_commitment_constants.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async saveValidators(validatorsWithIndices: { index: number; data: any }[], epoch: bigint) {
    await this.waitForConnection();

    const args = await Promise.all(validatorsWithIndices.map(async (validator) => {
      await this.addToEpochLookup(`${validator_commitment_constants.validatorKey}:${validator.index}`, epoch);
      return {
        key: `${validator_commitment_constants.validatorKey}:${validator.index}:${epoch}`,
        path: '$',
        value: validator.data,
      };
    }));

    await this.redisClient.json.mSet(args);
  }

  async saveValidatorBalancesInput(
    inputsWithIndices: { index: number; input: any }[],
  ) {
    await this.waitForConnection();

    const args = inputsWithIndices.map(ii => ({
      key: `${validator_commitment_constants.validatorBalanceInputKey}:${ii.index}`,
      path: '$',
      value: ii.input,
    }));

    await this.redisClient.json.mSet(args);
  }

  async saveFinalProofInput(input: {
    stateRoot: number[];
    slot: string;
    slotBranch: number[][];
    withdrawalCredentials: string;
    balanceBranch: number[][];
    validatorsBranch: number[][];
    validatorsSizeBits: number[];
  }) {
    await this.waitForConnection();

    await this.redisClient.json.set(
      validator_commitment_constants.finalProofInputKey,
      "$",
      input as any
    );
  }

  async saveValidatorProof(
    gindex: bigint,
    epoch: bigint,
    proof: ValidatorProof = {
      needsChange: true,
      proof: [],
      poseidonHash: [],
      sha256Hash: [],
    },
  ): Promise<void> {
    await this.waitForConnection();
    await this.redisClient.json.set(`${validator_commitment_constants.validatorProofKey}:${gindex}:${epoch}`, "$", proof as any);
  }

  async saveBalanceProof(
    depth: bigint,
    index: bigint,
    proof: BalanceProof = {
      needsChange: true,
      rangeTotalValue: '0',
      validatorsCommitment: [],
      proof: [],
      balancesHash: [],
      withdrawalCredentials: '0',
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.redisClient.set(
      `${validator_commitment_constants.balanceVerificationProofKey
      }:${depth.toString()}:${index.toString()}`,
      JSON.stringify(proof),
    );
  }

  async getNextProof(slot: number): Promise<ProofResultType | null> {
    await this.waitForConnection();

    const keys = await this.redisClient.keys(`proof:${slot}:*`);

    if (keys.length == 0) {
      return null;
    }

    return JSON.parse((await this.redisClient.get(keys[0]))!);
  }

  async getProof(
    prevSlot: number,
    nextSlot: number,
  ): Promise<ProofResultType | null> {
    await this.waitForConnection();

    let proof = await this.redisClient.get(`proof:${prevSlot}:${nextSlot}`);

    if (proof == null) {
      return null;
    }

    return JSON.parse(proof);
  }

  async get(key: string): Promise<string | null> {
    await this.waitForConnection();

    return await this.redisClient.get(key);
  }

  async set(key: string, value: string): Promise<void> {
    await this.waitForConnection();

    await this.redisClient.set(key, value);
  }

  async saveProof(
    prevSlot: number,
    nextSlot: number,
    proof: ProofResultType,
  ): Promise<void> {
    await this.waitForConnection();

    await this.redisClient.json.set(
      `proof:${prevSlot}:${nextSlot}`,
      '$',
      proof as any,
    );
  }

  async subscribeForProofs(
    listener: (message: string, channel: string) => unknown,
  ): Promise<void> {
    await this.waitForConnection();

    await this.pubSub.subscribe('proofs_channel', listener);
  }

  async getEpochsCount(gindex: number): Promise<number> {
    await this.waitForConnection();

    const result = await this.redisClient.keys(
      `${validator_commitment_constants.validatorProofKey}:${gindex}:*`,
    );

    if (result == null) {
      return 0;
    }

    return Number(result);
  }

  async getPathForEpoch(
    validatorIndex: number,
    epoch: number,
  ): Promise<ValidatorProof[]> {
    await this.waitForConnection();

    let gindex = 2 ** 40 - 1 + validatorIndex;

    let path: ValidatorProof[] = [];

    for (let i = 0; i < 40; i++) {
      let siblingGindex = getSiblingGindex(gindex);
      const changes = await this.redisClient.keys(
        `${validator_commitment_constants.validatorProofKey}:${siblingGindex}:*`,
      );

      if (changes.length == 0) {
        const level = Math.floor(Math.log2(gindex + 1));
        path.push(await this.getZeroPerLevel(level));
      } else {
        let lowerBoundEpoch = lowerBound(changes.map(Number), epoch);
        const commitmentMapping = JSON.parse((await this.redisClient.get(
          `${validator_commitment_constants.validatorProofKey}:${siblingGindex}:${lowerBoundEpoch}`
        ))!);
        path.push(commitmentMapping);
      }

      gindex = Math.floor((gindex - 1) / 2);
    }

    return path
  }

  private async getZeroPerLevel(level: number): Promise<ValidatorProof> {
    await this.waitForConnection();

    const result = await this.redisClient.get(
      `${validator_commitment_constants.zeroHashesForLevelKey}:${level}`,
    );

    return JSON.parse(result!);
  }

  private async waitForConnection() {
    if (!this.redisClient.isOpen) {
      await this.redisClient.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}

function getSiblingGindex(gindex: number): number {
  if (gindex % 2 == 0) {
    // node is right sibling
    return gindex - 1;
  } else {
    // node is left sibling
    return gindex + 1;
  }
}

function lowerBound(arr: number[], elem: number): number {
  let low = 0;
  let high = arr.length;
  while (low < high) {
    let mid = Math.floor((high + low) / 2);
    if (elem + 1 <= arr[mid]) {
      high = mid;
    } else {
      low = mid + 1;
    }
  }

  return arr[low - 1];
}
