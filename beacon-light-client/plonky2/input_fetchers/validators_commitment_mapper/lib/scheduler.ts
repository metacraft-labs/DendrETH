import {
  getDepthByGindex,
  getLastSlotInEpoch,
  gindexFromIndex,
  makeBranchIterator,
  sleep,
  splitIntoBatches,
} from '@dendreth/utils/ts-utils/common-utils';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  Validator,
  IndexedValidator,
  CommitmentMapperInput,
  ValidatorProof,
} from '@dendreth/relay/types/types';
import chalk from 'chalk';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../../../kv_db_constants.json';
import {
  commitmentMapperInputFromValidator,
  createDummyCommitmentMapperInput,
} from '../../utils/common_utils';
import { ChainableCommander } from 'ioredis';
import { validatorFromValidatorJSON } from '@dendreth/relay/utils/converters';
import _ from 'underscore';
import assert from 'assert';

enum TaskTag {
  HASH_CONCATENATION_PROOF = 0,
  DUMMY_PROOF_FOR_DEPTH = 1,
  HASH_VALIDATOR_PROOF = 2,
  ZERO_OUT_VALIDATOR = 3,
}

export class CommitmentMapperScheduler {
  private redis: Redis;
  private api: BeaconApi;
  private queues: WorkQueue[] = [];
  private currentSlot: bigint;
  private lastFinalizedEpoch: bigint;
  private headSlot: bigint;
  private take: number | undefined;
  private offset: number | undefined;
  private validators: Validator[] = [];

  async init(options: any): Promise<void> {
    this.api = await getBeaconApi(options['beacon-node']);
    this.redis = new Redis(
      options['redis-host'],
      options['redis-port'],
      options['redis-auth'],
    );

    for (let depth = 0; depth <= 40; ++depth) {
      const prefix = new KeyPrefix(
        `${CONSTANTS.validatorProofsQueue}:${depth}`,
      );
      const queue = new WorkQueue(prefix);
      this.queues.push(queue);
    }

    this.take = options['take'];
    this.offset = options['offset'];
    this.headSlot = await this.api.getHeadSlot();
    this.lastFinalizedEpoch = await this.api.getLastFinalizedCheckpoint();

    const lastProcessedSlot = await this.redis.get(
      CONSTANTS.lastProcessedSlotKey,
    );

    this.currentSlot =
      lastProcessedSlot !== null
        ? BigInt(lastProcessedSlot)
        : (() => {
          const finalizedSlot = getLastSlotInEpoch(this.lastFinalizedEpoch);
          const slot = options['sync-slot'] || finalizedSlot;
          return BigInt(Math.min(Number(slot), Number(finalizedSlot))) - 1n;
        })();

    const lastVerifiedSlot = await this.redis.get(
      CONSTANTS.lastVerifiedSlotKey,
    );

    if (lastVerifiedSlot === null) {
      await this.redis.set(
        CONSTANTS.lastVerifiedSlotKey,
        `${this.currentSlot}`,
      );
    }
  }

  dispose(): Promise<void> {
    return this.redis.quit();
  }

  async start(runOnce: boolean = false): Promise<void> {
    console.log(chalk.bold.blue('Fetching validators from database...'));
    this.validators = await getValidatorsBatched(this.redis, this.currentSlot);

    await this.ensureDummyProofs(this.currentSlot);

    if (runOnce) {
      this.currentSlot++;
      console.log(chalk.bold.blue(`Processing slot ${this.currentSlot}`));
      await this.pushDataForCurrentSlot();
      return;
    }

    await this.syncToHeadSlot();
    this.registerEventListeners();

    while (true) {
      await this.syncToHeadSlot();
      await sleep(5000);
    }
  }

  async syncToHeadSlot(): Promise<void> {
    while (this.currentSlot < this.headSlot) {
      this.currentSlot++;
      logProgress(this.currentSlot, this.headSlot);
      await this.pushDataForCurrentSlot();
    }
  }

  async pushDataForCurrentSlot(): Promise<void> {
    const pipeline = this.redis.client.pipeline();

    await this.updateValidators(pipeline);
    setValidatorsLengthForSlot(
      pipeline,
      this.currentSlot,
      this.validators.length,
    );
    updateLastProcessedSlot(pipeline, this.currentSlot);
    await pipeline.exec();
  }

  /// Updates the validators tree for the new slot. The operation is atomic if below a
  /// certain threshold of changes. Otherwise a long redis command string is
  /// created that can't be handled by js. The operation should be atomic every
  /// time except on the first run.
  async updateValidators(pipeline: ChainableCommander): Promise<void> {
    const changedValidators = await this.getValidatorsDiff();
    this.applyValidatorsDiff(changedValidators);

    if (changedValidators.length <= 1000) {
      await this.modifyValidatorsPipeline(
        pipeline,
        changedValidators,
        this.currentSlot,
      );
    } else {
      await this.modifyValidators(changedValidators, this.currentSlot);
    }

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );
  }

  scheduleDummyProofTasks(pipeline: ChainableCommander, slot: bigint): void {
    saveDummyInput(pipeline, this.currentSlot);
    saveDummyProofPlaceholder(pipeline, 40n);
    this.pushHashValidatorProofTask(
      pipeline,
      BigInt(CONSTANTS.validatorRegistryLimit),
      slot,
    );

    for (let depth = 39n; depth >= 0n; depth--) {
      saveDummyProofPlaceholder(pipeline, depth);
      this.scheduleDummyProofForDepth(pipeline, depth);
    }
  }

  async scheduleDummyProofForDepth(
    pipeline: ChainableCommander,
    depth: bigint,
  ): Promise<void> {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.DUMMY_PROOF_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    const item = new Item(Buffer.from(buffer));
    this.pushQueueItemPipeline(pipeline, item, Number(depth));
  }

  scheduleZeroOutValidatorTask(
    pipeline: ChainableCommander,
    validatorIndex: number,
    slot: bigint,
  ): void {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.ZERO_OUT_VALIDATOR);
    dataView.setBigUint64(1, BigInt(validatorIndex), false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.pushQueueItemPipeline(pipeline, item, 40);
  }

  modifyValidators = modifyValidatorsImpl({
    scheduleHashValidatorTaskFn: this.scheduleHashValidatorProofTask.bind(this),
    scheduleHashConcatenationTaskFn:
      this.scheduleHashConcatenationTask.bind(this),
  });

  modifyValidatorsPipeline(
    pipeline: ChainableCommander,
    ...args: Parameters<ReturnType<typeof modifyValidatorsImpl>>
  ): Promise<void> {
    return modifyValidatorsImpl({
      scheduleHashValidatorTaskFn: async (...args) =>
        this.scheduleHashValidatorTaskPipeline.bind(this, pipeline)(...args),
      scheduleHashConcatenationTaskFn: async (...args) =>
        this.scheduleHashConcatenationTaskPipeline.bind(
          this,
          pipeline,
        )(...args),
    })(...args);
  }

  async ensureDummyProofs(slot: bigint): Promise<void> {
    const pipeline = this.redis.client.pipeline();

    const isInitialized = await this.isInitialized();
    if (!isInitialized) {
      console.log(chalk.bold.blue('Scheduling dummy proof tasks...'));
      this.scheduleDummyProofTasks(pipeline, slot);
    }
    await pipeline.exec();
  }

  async isInitialized(): Promise<boolean> {
    const result = await this.redis.client.exists(
      `${CONSTANTS.validatorProofKey}:zeroes:1`,
    );
    return result === 1;
  }

  registerEventListeners(): void {
    const eventSource = this.api.subscribeForEvents([
      'head',
      'finalized_checkpoint',
    ]);

    eventSource.addEventListener('head', (event: any) => {
      const slot = BigInt(JSON.parse(event.data).slot);
      if (slot > this.headSlot) {
        this.headSlot = slot;
      }
    });

    eventSource.addEventListener('finalized_checkpoint', (event: any) => {
      const epoch = BigInt(JSON.parse(event.data).epoch);
      if (epoch > this.lastFinalizedEpoch) {
        this.lastFinalizedEpoch = epoch;
      }
    });
  }

  pushHashValidatorProofTask(
    pipeline: ChainableCommander,
    validatorIndex: bigint,
    slot: bigint,
  ): void {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.HASH_VALIDATOR_PROOF);
    dataView.setBigUint64(1, BigInt(validatorIndex), false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.pushQueueItemPipeline(pipeline, item, 40);
  }

  scheduleHashValidatorTaskPipeline(
    pipeline: ChainableCommander,
    indexedValidator: IndexedValidator,
    slot: bigint,
  ): void {
    saveRealInput(pipeline, indexedValidator, slot);
    this.pushHashValidatorProofTask(
      pipeline,
      BigInt(indexedValidator.index),
      slot,
    );

    const gindex = gindexFromIndex(BigInt(indexedValidator.index), 40n);
    saveProofPlaceholder(pipeline, gindex, slot);
    recordStateModification(
      pipeline,
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      slot,
    );
  }

  scheduleHashValidatorProofTask(
    indexedValidator: IndexedValidator,
    slot: bigint,
  ): Promise<unknown> {
    const pipeline = this.redis.client.pipeline();
    this.scheduleHashValidatorTaskPipeline(pipeline, indexedValidator, slot);
    return pipeline.exec();
  }

  scheduleHashConcatenationTaskPipeline(
    pipeline: ChainableCommander,
    gindex: bigint,
    slot: bigint,
  ): void {
    recordProofStateModification(pipeline, gindex, slot);
    saveProofPlaceholder(pipeline, gindex, slot);

    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.HASH_CONCATENATION_PROOF);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.pushQueueItemPipeline(
      pipeline,
      item,
      getDepthByGindex(Number(gindex)),
    );
  }

  async scheduleHashConcatenationTask(
    gindex: bigint,
    slot: bigint,
  ): Promise<unknown> {
    const pipeline = this.redis.client.pipeline();
    this.scheduleHashConcatenationTaskPipeline(pipeline, gindex, slot);
    return pipeline.exec();
  }

  async getValidatorsDiff(): Promise<IndexedValidator[]> {
    const validatorsAreInitialized =
      (await this.redis.client.exists(
        `${CONSTANTS.validatorKey}:0:slot_lookup`,
      )) !== 0;

    if (validatorsAreInitialized) {
      return this.api.getValidatorsDiffCustomEndpoint(
        this.currentSlot,
        this.offset,
        this.take,
      );
    } else {
      const newValidators = await this.api.getValidators(
        this.currentSlot,
        this.take,
        this.offset,
      );

      return newValidators
        .map((validator, index) => ({ validator, index }))
        .filter(hasValidatorChanged(this.validators));
    }
  }

  applyValidatorsDiff(diff: IndexedValidator[]): void {
    diff.forEach(({ index, validator }) => {
      this.validators[index] = validator;
    });
  }

  pushQueueItemPipeline(
    pipeline: ChainableCommander,
    item: Item,
    depth: number,
  ) {
    assert(depth <= 40, 'The validators tree is 40 levels deep');
    this.queues[depth].addItemToPipeline(pipeline, item);
  }
}

interface ModifyValidatorsVTable {
  scheduleHashValidatorTaskFn: (
    indexedValidator: IndexedValidator,
    slot: bigint,
  ) => Promise<unknown>;
  scheduleHashConcatenationTaskFn: (
    gindex: bigint,
    slot: bigint,
  ) => Promise<unknown>;
}

/// A template for validators tree modification (currently used to create
/// two implementations. One that uses a pipeline and one that executes each
/// command separately)
function modifyValidatorsImpl({
  scheduleHashValidatorTaskFn,
  scheduleHashConcatenationTaskFn,
}: ModifyValidatorsVTable) {
  return async function(indexedValidators: IndexedValidator[], slot: bigint) {
    await Promise.all(
      indexedValidators.map(indexedValidator => {
        return scheduleHashValidatorTaskFn(indexedValidator, slot);
      }),
    );

    const validatorIndices = indexedValidators.map(x => x.index);
    let levelIterator = makeBranchIterator(validatorIndices.map(BigInt), 40n);

    // ignore leaves
    levelIterator.next();

    for (const gindices of levelIterator) {
      await Promise.all(
        gindices.map(gindex => scheduleHashConcatenationTaskFn(gindex, slot)),
      );
    }
  };
}

function logProgress(currentSlot: bigint, headSlot: bigint): void {
  const progressMessage =
    currentSlot === headSlot
      ? chalk.cyan(currentSlot)
      : `${chalk.cyanBright(currentSlot)}/${chalk.cyan(headSlot)}`;

  console.log(chalk.bold.blue(`Syncing ${progressMessage}...`));
}

export async function setValidatorsLengthForSlot(
  pipeline: ChainableCommander,
  slot: bigint,
  length: number,
): Promise<void> {
  pipeline.set(`${CONSTANTS.validatorsLengthKey}:${slot}`, length.toString());
}

function updateLastProcessedSlot(
  pipeline: ChainableCommander,
  slot: bigint,
): void {
  pipeline.set(CONSTANTS.lastProcessedSlotKey, slot.toString());
}

function recordStateModification(
  pipeline: ChainableCommander,
  key: string,
  slot: bigint,
): void {
  pipeline.zadd(
    `${key}:${CONSTANTS.slotLookupKey}`,
    Number(slot),
    slot.toString(),
  );
}

function recordValidatorStateModification(
  pipeline: ChainableCommander,
  index: bigint,
  slot: bigint,
): void {
  recordStateModification(pipeline, `${CONSTANTS.validatorKey}:${index}`, slot);
}

function recordProofStateModification(
  pipeline: ChainableCommander,
  gindex: bigint,
  slot: bigint,
): void {
  recordStateModification(
    pipeline,
    `${CONSTANTS.validatorProofKey}:${gindex}`,
    slot,
  );
}

function saveInput(
  pipeline: ChainableCommander,
  index: bigint,
  input: CommitmentMapperInput,
  slot: bigint,
): void {
  recordValidatorStateModification(pipeline, index, slot);

  pipeline.set(
    `${CONSTANTS.validatorKey}:${index}:${slot}`,
    JSON.stringify(input),
  );
}

function saveRealInput(
  pipeline: ChainableCommander,
  { validator, index }: IndexedValidator,
  slot: bigint,
): void {
  const input = commitmentMapperInputFromValidator(validator);
  saveInput(pipeline, BigInt(index), input, slot);
}

function saveDummyInput(pipeline: ChainableCommander, slot: bigint): void {
  const index = BigInt(CONSTANTS.validatorRegistryLimit);
  const input = createDummyCommitmentMapperInput();
  saveInput(pipeline, index, input, slot);
}

async function saveDummyProofPlaceholder(
  pipeline: ChainableCommander,
  depth: bigint,
): Promise<void> {
  const obj: ValidatorProof = {
    needsChange: true,
    proofKey: 'invalid',
    publicInputs: {
      poseidonHashTreeRoot: [0, 0, 0, 0],
      sha256HashTreeRoot: ''.padEnd(64, '0'),
    },
  };

  pipeline.set(
    `${CONSTANTS.validatorProofKey}:zeroes:${depth}`,
    JSON.stringify(obj),
  );
}

function saveProofPlaceholder(
  pipeline: ChainableCommander,
  gindex: bigint,
  slot: bigint,
): void {
  const obj: ValidatorProof = {
    needsChange: true,
    proofKey: '',
    publicInputs: {
      poseidonHashTreeRoot: [0, 0, 0, 0],
      sha256HashTreeRoot: ''.padEnd(64, '0'),
    },
  };

  pipeline.set(
    `${CONSTANTS.validatorProofKey}:${gindex}:${slot}`,
    JSON.stringify(obj),
  );
}

async function getValidatorKeysForSlot(
  redis: Redis,
  slot: bigint,
): Promise<string[]> {
  return (await redis.client.keys(`${CONSTANTS.validatorKey}:*:[0-9]*`))
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

async function getValidatorsBatched(
  redis: Redis,
  slot: bigint,
  batchSize = 1000,
): Promise<Validator[]> {
  const keys = await getValidatorKeysForSlot(redis, slot);
  let allValidators: Validator[] = new Array(keys.length);

  for (const [keyBatchIndex, batchKeys] of splitIntoBatches(
    keys,
    batchSize,
  ).entries()) {
    const res = await redis.client.mget(batchKeys);
    if (res === null) {
      continue;
    }
    const batchValidators = (
      res.filter((v: any) => v !== null) as string[]
    ).map((json: any) => JSON.parse(json).validator);

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

// function updateLastVerifiedSlot(pipeline: ChainableCommander, slot: bigint): void {
//   pipeline.set(CONSTANTS.lastVerifiedSlotKey, slot.toString());
// }

// Returns a function that checks whether a validator at validator index has changed
function hasValidatorChanged(prevValidators: Validator[]) {
  return ({ validator, index }: IndexedValidator): boolean =>
    prevValidators[index] === undefined ||
    validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
    validator.slashed !== prevValidators[index].slashed ||
    validator.activationEligibilityEpoch !==
    prevValidators[index].activationEligibilityEpoch ||
    validator.activationEpoch !== prevValidators[index].activationEpoch ||
    validator.exitEpoch !== prevValidators[index].exitEpoch ||
    validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch ||
    _.zip(
      prevValidators[index].withdrawalCredentials,
      validator.withdrawalCredentials,
    ).some(([a, b]) => a !== b);
}
