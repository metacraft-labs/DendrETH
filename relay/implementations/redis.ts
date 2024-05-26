import { IRedis } from '../abstraction/redis-interface';
import {
  BalanceProof,
  ProofResultType,
  Validator,
  ValidatorProof,
  BlsDepositData,
  BalancesAccumulatorInput,
  CommitmentMapperInput,
} from '../types/types';
import { RedisClientType, createClient } from 'redis';
import CONSTANTS from '../../beacon-light-client/plonky2/kv_db_constants.json';
// TODO: move this to @dendreth/utils
import { getDepthByGindex } from '../../beacon-light-client/plonky2/input_fetchers/utils/common_utils';
import { Redis as RedisClient } from 'ioredis';
import chalk from 'chalk';
import { splitIntoBatches } from '@dendreth/utils/ts-utils/common-utils';
import { validatorFromValidatorJSON } from '../utils/converters';

export class Redis implements IRedis {
  public readonly client: RedisClient;
  private readonly pubSub: RedisClientType;

  constructor(redisHost: string, redisPort: number) {
    this.client = new RedisClient({
      host: redisHost,
      port: redisPort,
    });

    this.pubSub = createClient({
      url: `redis://${redisHost}:${redisPort}`,
    });
  }

  async quit() {
    await this.waitForConnection();
    await this.pubSub.quit();
    this.client.quit();
  }

  async addToSlotLookup(key: string, slot: bigint) {
    await this.waitForConnection();

    await this.client.zadd(
      `${key}:${CONSTANTS.slotLookupKey}`,
      Number(slot),
      slot.toString(),
    );
  }

  async removeFromSlotLookup(key: string, ...slots: bigint[]) {
    await this.waitForConnection();

    await this.client.zrem(
      `${key}:${CONSTANTS.slotLookupKey}`,
      slots.map(String),
    );
  }

  async getSlotWithLatestChange(
    key: string,
    slot: bigint,
  ): Promise<bigint | null> {
    await this.waitForConnection();

    const values = await this.client.zrange(
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

  async collectOutdatedSlots(
    key: string,
    newOldestSlot: bigint,
  ): Promise<bigint[]> {
    await this.waitForConnection();

    const slotWithLatestChange = await this.getSlotWithLatestChange(
      key,
      newOldestSlot,
    );
    if (slotWithLatestChange !== null) {
      return (
        await this.client.zrange(
          `${key}:${CONSTANTS.slotLookupKey}`,
          0,
          (slotWithLatestChange - 1n).toString(),
          'BYSCORE',
        )
      ).map(BigInt);
    }
    return [];
  }

  async pruneOldSlots(key: string, newOldestSlot: bigint): Promise<number> {
    await this.waitForConnection();

    const slots = await this.collectOutdatedSlots(key, newOldestSlot);
    if (slots.length !== 0) {
      await this.removeFromSlotLookup(key, ...slots);
    }
    return 0;
  }

  async updateLastVerifiedSlot(slot: bigint) {
    await this.waitForConnection();

    this.client.set(CONSTANTS.lastVerifiedSlotKey, slot.toString());
  }

  async updateLastProcessedSlot(slot: bigint) {
    await this.waitForConnection();

    this.client.set(CONSTANTS.lastProcessedSlotKey, slot.toString());
  }

  async getAllKeys(pattern: string): Promise<string[]> {
    await this.waitForConnection();
    return this.client.keys(pattern);
  }

  async extractHashFromCommitmentMapperProof(
    gindex: bigint,
    slot: bigint,
    hashAlgorithm: 'sha256' | 'poseidon',
  ): Promise<number[] | null> {
    const hashAlgorithmOptionMap = {
      sha256: 'sha256Hash',
      poseidon: 'poseidonHash',
    };

    const hashKey = hashAlgorithmOptionMap[hashAlgorithm];

    const latestSlot = await this.getSlotWithLatestChange(
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      slot,
    );
    if (latestSlot === null) {
      const depth = getDepthByGindex(Number(gindex));
      const result = await this.client.get(
        `${CONSTANTS.validatorProofKey}:zeroes:${depth}`,
      );

      if (result == null) {
        return null;
      }

      return JSON.parse(result)[hashKey];
    }

    const key = `${CONSTANTS.validatorProofKey}:${gindex}:${latestSlot}`;
    const result = await this.client.get(key);
    if (result == null) {
      return null;
    }
    return JSON.parse(result)[hashKey];
  }

  async getValidatorsRoot(slot: bigint): Promise<String | null> {
    return this.client.get(`${CONSTANTS.validatorsRootKey}:${slot}`);
  }

  async deleteValidatorsRoot(slot: bigint): Promise<void> {
    await this.client.del(`${CONSTANTS.validatorsRootKey}:${slot}`);
  }

  async getValidatorsCommitmentRoot(slot: bigint): Promise<string[] | null> {
    const prefix = `${CONSTANTS.validatorProofKey}:1`;
    const latestRootChangeSlot = await this.getSlotWithLatestChange(
      prefix,
      slot,
    );

    if (latestRootChangeSlot == null) return null;

    const rootData = await this.client.get(`${prefix}:${latestRootChangeSlot}`);
    if (rootData == null) return null;

    const obj = JSON.parse(rootData);
    const poseidonHash = obj.poseidonHash;
    return poseidonHash;
  }

  async notifyAboutNewProof(): Promise<void> {
    await this.waitForConnection();

    this.pubSub.publish('proofs_channel', 'proof');
  }

  async getValidatorsLengthForSlot(slot: bigint): Promise<number | null> {
    const result = await this.get(`${CONSTANTS.validatorsLengthKey}:${slot}`);
    return result !== null ? Number(result) : null;
  }

  async getValidatorKeysForSlot(slot: bigint): Promise<string[]> {
    return (await this.client.keys(`${CONSTANTS.validatorKey}:*:[0-9]*`))
      .filter(key => !key.includes(CONSTANTS.validatorRegistryLimit.toString()))
      .reduce((acc, key) => {
        const split = key.split(':');
        const index = Number(split[1]);
        const keySlot = Number(split[2]);

        let latestSlot = 0;
        if (keySlot <= slot) {
          latestSlot = keySlot;
        }

        if (acc[index] && acc[index] > latestSlot) {
          latestSlot = acc[index];
        }

        acc[index] = latestSlot;
        return acc;
      }, new Array())
      .map((slot, index) => `${CONSTANTS.validatorKey}:${index}:${slot}`);
  }

  async getValidatorsBatched(
    slot: bigint,
    batchSize = 1000,
  ): Promise<Validator[]> {
    await this.waitForConnection();

    const keys = await this.getValidatorKeysForSlot(slot);
    let allValidators: Validator[] = new Array(keys.length);

    for (const [keyBatchIndex, batchKeys] of splitIntoBatches(
      keys,
      batchSize,
    ).entries()) {
      const res = await this.client.mget(batchKeys);
      if (res === null) {
        continue;
      }
      const batchValidators = (res.filter(v => v !== null) as string[]).map(
        (json: any) => JSON.parse(json).validator,
      );

      for (const [index, redisValidator] of batchValidators.entries()) {
        try {
          const validator = validatorFromValidatorJSON(redisValidator);
          const validatorIndex = Number(batchKeys[index].split(':')[1]);
          allValidators[validatorIndex] = validator;
        } catch (e) {
          console.error(e);
          continue;
        }
      }
      console.log(
        `Loaded batch ${chalk.bold.yellowBright(
          keyBatchIndex + 1,
        )} /${chalk.bold.yellow(Math.ceil(keys.length / batchSize))}`,
      );
    }

    return allValidators;
  }

  async isZeroValidatorEmpty() {
    await this.waitForConnection();

    const result = await this.client.keys(
      `${CONSTANTS.validatorKey}:${CONSTANTS.validatorRegistryLimit}:*`,
    );

    return result.length === 0;
  }

  async isZeroBalanceEmpty() {
    await this.waitForConnection();

    const result = await this.client.get(
      `${CONSTANTS.validatorBalanceInputKey}:${CONSTANTS.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async saveValidators(
    validatorsWithIndices: { index: number; data: CommitmentMapperInput }[],
    slot: bigint,
  ) {
    await this.waitForConnection();

    const args = (
      await Promise.all(
        validatorsWithIndices.map(async validator => {
          await this.addToSlotLookup(
            `${CONSTANTS.validatorKey}:${validator.index}`,
            slot,
          );
          return [
            `${CONSTANTS.validatorKey}:${validator.index}:${slot}`,
            JSON.stringify(validator.data),
          ];
        }),
      )
    ).flat();

    await this.client.mset(...args);
  }

  async saveBalancesAccumulatorProof(
    protocol: string,
    level: bigint,
    index: bigint,
    proof: BalanceProof = {
      needsChange: true,
      proofKey: '',
      rangeTotalValue: '0',
      validatorsCommitment: [],
      balancesHash: [],
      withdrawalCredentials: [],
      currentEpoch: '0',
      numberOfNonActivatedValidators: 0,
      numberOfActiveValidators: 0,
      numberOfExitedValidators: 0,
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `${CONSTANTS.balanceVerificationAccumulatorProofKey}:${protocol}:${level}:${index}`,
      JSON.stringify(proof),
    );
  }

  async saveBalancesAccumulatorInput(
    balancesInputs: BalancesAccumulatorInput[],
    protocol: string,
  ) {
    await this.waitForConnection();

    const args = balancesInputs
      .map((input, index) => {
        return [
          `${CONSTANTS.balanceVerificationAccumulatorKey}:${protocol}:${index}`,
          JSON.stringify(input),
        ];
      })
      .flat();

    await this.client.mset(...args);
  }

  async saveValidatorBalancesInput(
    protocol: string,
    inputsWithIndices: { index: number; input: any }[],
  ) {
    await this.waitForConnection();

    const args = inputsWithIndices
      .map(ii => {
        return [
          `${protocol}:${CONSTANTS.validatorBalanceInputKey}:${ii.index}`,
          JSON.stringify(ii.input),
        ];
      })
      .flat();

    await this.client.mset(...args);
  }

  async saveFinalProofInput(
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
    await this.waitForConnection();

    await this.client.set(
      `${protocol}:${CONSTANTS.finalProofInputKey}`,
      JSON.stringify(input),
    );
  }

  async saveValidatorProof(
    gindex: bigint,
    slot: bigint,
    proof: ValidatorProof = {
      needsChange: true,
      proofKey: '',
      poseidonHash: [],
      sha256Hash: [],
    },
  ): Promise<void> {
    await this.waitForConnection();
    await this.client.set(
      `${CONSTANTS.validatorProofKey}:${gindex}:${slot}`,
      JSON.stringify(proof),
    );
  }

  async saveZeroValidatorProof(
    depth: bigint,
    proof: ValidatorProof = {
      needsChange: true,
      proofKey: 'invalid',
      poseidonHash: [],
      sha256Hash: [],
    },
  ): Promise<void> {
    await this.waitForConnection();
    await this.client.set(
      `${CONSTANTS.validatorProofKey}:zeroes:${depth}`,
      JSON.stringify(proof),
    );
  }

  async saveBalanceProof(
    protocol: string,
    level: bigint,
    index: bigint,
    proof: BalanceProof = {
      needsChange: true,
      rangeTotalValue: '0',
      validatorsCommitment: [],
      proofKey: '',
      balancesHash: [],
      withdrawalCredentials: [],
      currentEpoch: '0',
      numberOfNonActivatedValidators: 0,
      numberOfActiveValidators: 0,
      numberOfExitedValidators: 0,
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `${protocol}:${CONSTANTS.balanceVerificationProofKey}:${level}:${index}`,
      JSON.stringify(proof),
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

  public async setValidatorsLength(slot: bigint, length: number) {
    await this.waitForConnection();
    await this.client.set(
      `${CONSTANTS.validatorsLengthKey}:${slot}`,
      length.toString(),
    );
  }

  async getDepositsCount(): Promise<number> {
    await this.waitForConnection();

    const pattern = `${CONSTANTS.depositSignatureVerificationKey}:*`;
    const keys = await this.client.keys(pattern);

    return keys.length;
  }

  async saveDeposit(index: number, deposit: BlsDepositData): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `${CONSTANTS.depositSignatureVerificationKey}:${index}`,
      JSON.stringify(deposit),
    );
  }

  async get(key: string): Promise<string | null> {
    await this.waitForConnection();
    return this.client.get(key);
  }

  async getBuffer(key: string): Promise<Buffer | null> {
    await this.waitForConnection();
    return this.client.getBuffer(key);
  }

  async setBuffer(key: string, buffer: Buffer): Promise<void> {
    await this.waitForConnection();
    await this.client.set(key, buffer);
  }

  async getBalanceWrapperProofWithPublicInputs(protocol: string): Promise<any> {
    await this.waitForConnection();

    return this.client.get(
      `${protocol}:balance_wrapper_proof_with_public_inputs`,
    );
  }

  async getBalanceWrapperVerifierOnly(): Promise<any> {
    await this.waitForConnection();

    return this.client.get('balance_wrapper_verifier_only');
  }

  async set(key: string, value: string): Promise<void> {
    await this.waitForConnection();
    await this.client.set(key, value);
  }

  async del(key: string): Promise<number> {
    await this.waitForConnection();
    return this.client.del(key);
  }

  async saveProof(
    prevSlot: number,
    nextSlot: number,
    proof: ProofResultType,
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
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

  async subscribeForGnarkProofs(
    protocol: string,
    listener: (message: string, channel: string) => unknown,
  ): Promise<void> {
    await this.waitForConnection();

    await this.pubSub.subscribe(`${protocol}:gnark_proofs_channel`, listener);
  }

  private async waitForConnection() {
    if (!['connect', 'connecting', 'ready'].includes(this.client.status)) {
      await this.client.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}
