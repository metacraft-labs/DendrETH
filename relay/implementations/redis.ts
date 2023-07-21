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

  async notifyAboutNewProof(): Promise<void> {
    await this.waitForConnection();

    this.pubSub.publish('proofs_channel', 'proof');
  }

  async getValidatorsBatched(ssz, batchSize = 1000) {
    await this.waitForConnection();

    const keys = await this.redisClient.keys(
      `${validator_commitment_constants.validatorKey}:*`,
    );

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
            pubkey: hexToBytes(bitsToBytes(redisValidatorJSON.pubkey)),
            withdrawalCredentials: hexToBytes(
              bitsToBytes(redisValidatorJSON.withdrawalCredentials),
            ),
            effectiveBalance:
              ssz.phase0.Validator.fields.effectiveBalance.deserialize(
                hexToBytes(
                  bitsToBytes(redisValidatorJSON.effectiveBalance),
                ).slice(0, 8),
              ),

            slashed: ssz.phase0.Validator.fields.slashed.deserialize(
              hexToBytes(bitsToBytes(redisValidatorJSON.slashed)).slice(0, 1),
            ),
            activationEligibilityEpoch:
              ssz.phase0.Validator.fields.activationEligibilityEpoch.deserialize(
                hexToBytes(
                  bitsToBytes(redisValidatorJSON.activationEligibilityEpoch),
                ).slice(0, 8),
              ),
            activationEpoch:
              ssz.phase0.Validator.fields.activationEpoch.deserialize(
                hexToBytes(
                  bitsToBytes(redisValidatorJSON.activationEpoch),
                ).slice(0, 8),
              ),
            exitEpoch: ssz.phase0.Validator.fields.exitEpoch.deserialize(
              hexToBytes(bitsToBytes(redisValidatorJSON.exitEpoch)).slice(0, 8),
            ),
            withdrawableEpoch:
              ssz.phase0.Validator.fields.withdrawableEpoch.deserialize(
                hexToBytes(
                  bitsToBytes(redisValidatorJSON.withdrawableEpoch),
                ).slice(0, 8),
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

  async saveValidators(
    validatorsWithIndices: { index: number; validator: string }[],
  ) {
    await this.waitForConnection();
    const result: [string, string][] = validatorsWithIndices.map(vi => [
      `${validator_commitment_constants.validatorKey}:${vi.index}`,
      vi.validator,
    ]);

    await this.redisClient.mSet(result);
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

  async saveValidatorProof(
    depth: bigint,
    index: bigint,
    proof: ValidatorProof = {
      needsChange: true,
      proof: [],
      poseidonHash: [],
      sha256Hash: [],
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.redisClient.set(
      `${
        validator_commitment_constants.validatorProofKey
      }:${depth.toString()}:${index.toString()}`,
      JSON.stringify(proof),
    );
  }

  async saveBalanceProof(
    depth: bigint,
    index: bigint,
    proof: BalanceProof = {
      needsChange: true,
      rangeTotalValue: 0,
      validatorsCommitment: [],
      proof: [],
      balancesHash: [],
      withdrawalCredentials: [],
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.redisClient.set(
      `${
        validator_commitment_constants.balanceVerificationProofKey
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

  private async waitForConnection() {
    if (!this.redisClient.isOpen) {
      await this.redisClient.connect();
    }

    if (!this.pubSub.isOpen) {
      await this.pubSub.connect();
    }
  }
}
