import {
  splitIntoBatches,
} from '../../libs/typescript/ts-utils/common-utils';
import { hexToBytes } from '../../libs/typescript/ts-utils/bls';
import { IRedis } from '../abstraction/redis-interface';
import {
  BalanceProof,
  ProofResultType,
  Validator,
  ValidatorProof,
} from '../types/types';
import { RedisClientType, createClient } from 'redis';
import validator_commitment_constants from '../../beacon-light-client/plonky2/constants/validator_commitment_constants.json';
import RedisReJSON from 'ioredis-rejson';

export class Redis implements IRedis {
  public readonly client: RedisReJSON;
  private readonly pubSub: RedisClientType;

  constructor(redisHost: string, redisPort: number) {
    this.client = new RedisReJSON({
      host: redisHost,
      port: redisPort,
    });

    this.pubSub = createClient({
      url: `redis://${redisHost}:${redisPort}`,
    });
  }

  async disconnect() {
    await this.waitForConnection();
    await this.pubSub.disconnect();
    this.client.disconnect();
  }

  async addToEpochLookup(key: string, epoch: bigint) {
    await this.waitForConnection();

    await this.client.zadd(`${key}:${validator_commitment_constants.epochLookupKey}`, Number(epoch), epoch.toString());
  }

  async getLatestEpoch(key: string, epoch: bigint): Promise<bigint | null> {
    await this.waitForConnection();

    const values = await this.client.zrange(`${key}:${validator_commitment_constants.epochLookupKey}`, epoch.toString(), 0, 'BYSCORE', 'REV', 'LIMIT', 0, 1);
    if (values.length === 0) {
      return null;
    }
    return BigInt(values[0]);
  }

  async pruneOldEpochs(key: string, newOldestEpoch: bigint): Promise<number> {
    await this.waitForConnection();

    const latestEpoch = await this.getLatestEpoch(key, newOldestEpoch);
    if (latestEpoch !== null) {
      const range = await this.client.zrange(`${key}:${validator_commitment_constants.epochLookupKey}`, 0, (latestEpoch - 1n).toString(), 'BYSCORE');
      if (range.length !== 0) {
        await this.client.zrem(`${key}:${validator_commitment_constants.epochLookupKey}`, range);
        return await this.client.del(range.map((suffix) => `${key}:${suffix}`));
      }
    }
    return 0;
  }

  async getAllKeys(pattern: string): Promise<string[]> {
    await this.waitForConnection();
    return this.client.keys(pattern);
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
      const result = await this.client.json_get(`${validator_commitment_constants.validatorProofKey}:zeroes:${depth}`, hashKey) as any;
      return result;
    }

    const key = `${validator_commitment_constants.validatorProofKey}:${gindex}:${latestEpoch}`;
    return this.client.json_get(key, hashKey) as any;
  }

  async notifyAboutNewProof(): Promise<void> {
    await this.waitForConnection();

    this.pubSub.publish('proofs_channel', 'proof');
  }

  async getValidatorsBatched(ssz, batchSize = 1000): Promise<Validator[]> {
    await this.waitForConnection();

    const keys = (await this.client.keys(
      `${validator_commitment_constants.validatorKey}:*`,
    )).filter((key) => !key.includes(validator_commitment_constants.epochLookupKey));

    if (keys.length === 0) {
      return [];
    }

    let allValidators: Validator[] = new Array(keys.length);

    for (const [keyBatchIndex, batchKeys] of splitIntoBatches(keys, batchSize).entries()) {
      const res = await this.client.json_mget(batchKeys, '$');
      if (res === null) {
        continue;
      }
      const batchValidators = (res as any[]).filter((v) => v !== null).flat();

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
      console.log(`Loaded batch ${keyBatchIndex + 1}/${Math.ceil(keys.length / batchSize)}`);
    }

    return allValidators;
  }

  async isZeroValidatorEmpty() {
    await this.waitForConnection();

    const result = await this.client.keys(
      `${validator_commitment_constants.validatorKey}:${validator_commitment_constants.validatorRegistryLimit}:*`,
    );

    return result.length === 0;
  }

  async isZeroBalanceEmpty() {
    await this.waitForConnection();

    const result = await this.client.json_get(
      `${validator_commitment_constants.validatorBalanceInputKey}:${validator_commitment_constants.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async saveValidators(validatorsWithIndices: { index: number; data: any }[], epoch: bigint) {
    await this.waitForConnection();

    const args = await Promise.all(validatorsWithIndices.map(async (validator) => {
      await this.addToEpochLookup(`${validator_commitment_constants.validatorKey}:${validator.index}`, epoch);
      return [
        `${validator_commitment_constants.validatorKey}:${validator.index}:${epoch}`,
        '$',
        JSON.stringify(validator.data),
      ];
    }));

    await this.client.sendCommand(new RedisReJSON.Command('JSON.MSET', args));
  }

  async saveValidatorBalancesInput(
    inputsWithIndices: { index: number; input: any }[],
  ) {
    await this.waitForConnection();

    const args = inputsWithIndices.map(ii => ([
      `${validator_commitment_constants.validatorBalanceInputKey}:${ii.index}`,
      '$',
      JSON.stringify(ii.input),
    ]));

    await this.client.sendCommand(new RedisReJSON.Command('JSON.MSET', args));
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

    await this.client.json_set(
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
    await this.client.json_set(`${validator_commitment_constants.validatorProofKey}:${gindex}:${epoch}`, "$", proof as any);
  }

  async saveBalanceProof(
    level: bigint,
    index: bigint,
    proof: BalanceProof = {
      needsChange: true,
      rangeTotalValue: '0',
      validatorsCommitment: [],
      proof: [],
      balancesHash: [],
      withdrawalCredentials: '0',
      currentEpoch: '0',
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.json_set(
      `${validator_commitment_constants.balanceVerificationProofKey}:${level}:${index}`,
      '$',
      proof as any,
    );
  }

  async getNextProof(slot: number): Promise<ProofResultType | null> {
    await this.waitForConnection();

    const keys = await this.client.keys(`proof:${slot}:*`);

    if (keys.length == 0) {
      return null;
    }

    return JSON.parse((await this.client.get(keys[0]))!);
  }

  async getProof(
    prevSlot: number,
    nextSlot: number,
  ): Promise<ProofResultType | null> {
    await this.waitForConnection();

    let proof = await this.client.get(`proof:${prevSlot}:${nextSlot}`);

    if (proof == null) {
      return null;
    }

    return JSON.parse(proof);
  }

  async get(key: string): Promise<string | null> {
    await this.waitForConnection();

    return await this.client.get(key);
  }

  async set(key: string, value: string): Promise<void> {
    await this.waitForConnection();

    await this.client.set(key, value);
  }

  async saveProof(
    prevSlot: number,
    nextSlot: number,
    proof: ProofResultType,
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.json_set(
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

  private async waitForConnection() {
    if (this.client.status !== 'ready') {
      await this.client.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}
