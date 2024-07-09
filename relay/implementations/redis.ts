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
//
import { ChainableCommander, Redis as RedisClient, Result } from 'ioredis';
import chalk from 'chalk';
import {
  getDepthByGindex,
  splitIntoBatches,
} from '@dendreth/utils/ts-utils/common-utils';
import { validatorFromValidatorJSON } from '../utils/converters';
import JSONbig from 'json-bigint';

declare module 'ioredis' {
  interface RedisCommander<Context> {
    deletePattern(pattern: string): Result<string, Context>;
  }
}

function makeRedisURL(host: string, port: number, auth?: string): string {
  const at: string = (auth != null && auth.length > 0) ? `${auth}@` : "";
  return `redis://${at}${host}:${port}`;
}

export class Redis implements IRedis {
  public readonly client: RedisClient;
  private readonly pubSub: RedisClientType;

  constructor(redisHost: string, redisPort: number, redisAuth?: string) {
    const url: string = makeRedisURL(redisHost, redisPort, redisAuth);
    this.client = new RedisClient(url);
    this.pubSub = createClient({ url });

    this.client.defineCommand('deletePattern', {
      numberOfKeys: 0,
      lua: `
      local cursor = 0
      local calls = 0
      local dels = 0
      repeat
          local result = redis.call('SCAN', cursor, 'MATCH', ARGV[1])
          calls = calls + 1
          for _,key in ipairs(result[2]) do
              redis.call('DEL', key)
              dels = dels + 1
          end
          cursor = tonumber(result[1])
      until cursor == 0
      return "Calls " .. calls .. " Dels " .. dels
  `,
    });
  }

  async quit() {
    await this.waitForConnection();
    await this.pubSub.quit();
    this.client.quit();
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

  // async pruneOldSlots(key: string, newOldestSlot: bigint): Promise<number> {
  //   await this.waitForConnection();
  //
  //   const slots = await this.collectOutdatedSlots(key, newOldestSlot);
  //   if (slots.length !== 0) {
  //     await this.removeFromSlotLookup(key, ...slots);
  //   }
  //   return 0;
  // }


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
      sha256: 'sha256HashTreeRoot',
      poseidon: 'poseidonHashTreeRoot',
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

      return JSONbig.parse(result).publicInputs[hashKey];
    }

    const key = `${CONSTANTS.validatorProofKey}:${gindex}:${latestSlot}`;
    const result = await this.client.get(key);
    if (result == null) {
      return null;
    }

    return JSONbig.parse(result).publicInputs[hashKey];
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
        )}/${chalk.bold.yellow(Math.ceil(keys.length / batchSize))}`,
      );
    }

    console.log(
      `Loaded ${chalk.bold.yellow(
        allValidators.length,
      )} validators from database`,
    );

    return allValidators;
  }

  async isZeroBalanceEmpty() {
    await this.waitForConnection();

    const result = await this.client.get(
      `${CONSTANTS.validatorBalanceInputKey}:${CONSTANTS.validatorRegistryLimit}`,
    );

    return result == null;
  }

  async saveBalancesAccumulatorProof(
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

  async saveBalanceAggregatorFinalProofInput(
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
    await this.waitForConnection();

    await this.client.set(
      `${protocol}:${CONSTANTS.depositBalanceVerificationFinalProofInputKey}`,
      JSON.stringify(input),
    );
  }

  async saveBalanceProof(
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

  async extractHashFromDepositCommitmentMapperProof(
    protocol: string,
    gindex: bigint,
    hashAlgorithm: 'sha256' | 'poseidon',
  ): Promise<number[] | null> {
    const hashAlgorithmOptionMap = {
      sha256: 'sha256HashTreeRoot',
      poseidon: 'poseidonHashTreeRoot',
    };

    const hashKey = hashAlgorithmOptionMap[hashAlgorithm];

    const result = await this.client.get(
      `${protocol}:${CONSTANTS.balanceVerificationAccumulatorProofKey}:${gindex}`,
    );
    if (result === null) {
      const depth = getDepthByGindex(Number(gindex));
      const result = await this.client.get(
        `${protocol}:${CONSTANTS.balanceVerificationAccumulatorProofKey}:zeroes:${depth}`,
      );

      if (result == null) {
        return null;
      }

      return JSONbig.parse(result).publicInputs[hashKey];
    }

    return JSONbig.parse(result).publicInputs[hashKey];
  }

  async saveDepositBalanceVerificationInput(
    protocol: string,
    index: bigint,
    input: any,
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `${protocol}:${CONSTANTS.depositBalanceVerificationInputKey}:${index}`,
      JSONbig.stringify(input),
    );
  }

  async saveDepositBalanceVerificationProof(
    protocol: string,
    level: bigint,
    index: bigint,
    proof = {
      needsChange: true,
      proofKey: '',
      publicInputs: {
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
        validatorDeposit: {
          pubkey: ''.padEnd(96, '0'),
          depositIndex: '0',
          signature: ''.padEnd(192, '0'),
          depositMessageRoot: ''.padEnd(64, '0'),
        },
        commitmentMapperRoot: [''],
        commitmentMapperProof: [['']],
        validatorIndex: 0,
        validatorDepositRoot: [''],
        validatorDepositProof: [['']],
        balanceTreeRoot: ''.padEnd(64, '0'),
        balanceLeaf: ''.padEnd(64, '0'),
        balanceProof: [''.padEnd(64, '0')],
        currentEpoch: '0',
        isDummy: true,
      },
    },
  ): Promise<void> {
    await this.waitForConnection();

    await this.client.set(
      `${protocol}:${CONSTANTS.depositBalanceVerificationProofKey}:${level}:${index}`,
      JSONbig.stringify(proof),
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

  async getBalanceWrapperVerifierOnly(protocol: string): Promise<any> {
    await this.waitForConnection();

    return this.client.get(`${protocol}:balance_wrapper_verifier_only`);
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
