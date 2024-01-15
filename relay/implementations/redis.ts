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

    for (let i = 0; i < keys.length; i += batchSize) {
      const batchKeys = keys.slice(i, i + batchSize);
      const batchValidators = await this.redisClient.mGet(batchKeys);

      for (let j = 0; j < batchValidators.length; j++) {
        const redisValidatorJSON = JSON.parse(batchValidators[j]!);
        try {
          let validatorJSON: Validator = {
            pubkey: hexToBytes(redisValidatorJSON.pubkey),
            withdrawalCredentials: hexToBytes(
              redisValidatorJSON.withdrawalCredentials,
            ),
            effectiveBalance:
              ssz.phase0.Validator.fields.effectiveBalance.deserialize(
                hexToBytes(redisValidatorJSON.effectiveBalance).slice(0, 8),
              ),

            slashed: ssz.phase0.Validator.fields.slashed.deserialize(
              hexToBytes(redisValidatorJSON.slashed).slice(0, 1),
            ),
            activationEligibilityEpoch:
              ssz.phase0.Validator.fields.activationEligibilityEpoch.deserialize(
                hexToBytes(redisValidatorJSON.activationEligibilityEpoch).slice(
                  0,
                  8,
                ),
              ),
            activationEpoch:
              ssz.phase0.Validator.fields.activationEpoch.deserialize(
                hexToBytes(redisValidatorJSON.activationEpoch).slice(0, 8),
              ),
            exitEpoch: ssz.phase0.Validator.fields.exitEpoch.deserialize(
              hexToBytes(redisValidatorJSON.exitEpoch).slice(0, 8),
            ),
            withdrawableEpoch:
              ssz.phase0.Validator.fields.withdrawableEpoch.deserialize(
                hexToBytes(redisValidatorJSON.withdrawableEpoch).slice(0, 8),
              ),
          };

          const index = Number(batchKeys[j].split(':')[1]);

          allValidators[index] = validatorJSON;
        } catch (e) {
          console.log(e);
          continue;
        }
      }

      console.log(`Loaded batch, ${i / batchSize}/${keys.length / batchSize}`);
    }

    return allValidators;
  }

  async isZeroValidatorEmpty() {
    await this.waitForConnection();

    const result = await this.redisClient.get(
      `${validator_commitment_constants.validatorKey}:${validator_commitment_constants.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async isZeroBalanceEmpty() {
    await this.waitForConnection();

    const result = await this.redisClient.get(
      `${validator_commitment_constants.validatorBalanceInputKey}:${validator_commitment_constants.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async saveValidators(validatorsWithIndices: { index: number; validatorJSON: string }[], epoch: bigint) {
    await this.waitForConnection();

    const args: [string, string][] = await Promise.all(validatorsWithIndices.map(async (validator) => {
      await this.addToEpochLookup(`${validator_commitment_constants.validatorKey}:${validator.index}`, epoch);
      return [
        `${validator_commitment_constants.validatorKey}:${validator.index}:${epoch}`,
        validator.validatorJSON,
      ]
    }));

    await this.redisClient.mSet(args);
  }

  async saveValidatorBalancesInput(
    inputsWithIndices: { index: number; input: string }[],
  ) {
    await this.waitForConnection();

    const result: [string, string][] = inputsWithIndices.map(ii => [
      `${validator_commitment_constants.validatorBalanceInputKey}:${ii.index}`,
      ii.input,
    ]);

    await this.redisClient.mSet(result);
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

    await this.redisClient.set(
      validator_commitment_constants.finalProofInputKey,
      JSON.stringify(input),
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

    await this.redisClient.set(
      `${validator_commitment_constants.validatorProofKey
      }:${gindex}:${epoch}`,
      JSON.stringify(proof),
    );
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

    await this.redisClient.set(
      `proof:${prevSlot}:${nextSlot}`,
      JSON.stringify(proof),
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
