import {
  getLastSlotInEpoch,
  gindexFromIndex,
  makeBranchIterator,
  sleep,
} from '@dendreth/utils/ts-utils/common-utils';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { Validator, IndexedValidator, CommitmentMapperInput, ValidatorProof } from '@dendreth/relay/types/types';
import chalk from 'chalk';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../../../kv_db_constants.json';
import {
  commitmentMapperInputFromValidator,
  createDummyCommitmentMapperInput
} from '../../utils/common_utils';
import { ChainableCommander } from 'ioredis';

enum TaskTag {
  HASH_CONCATENATION_PROOF = 0,
  DUMMY_PROOF_FOR_DEPTH = 1,
  HASH_VALIDATOR_PROOF = 2,
  ZERO_OUT_VALIDATOR = 3,
}

export class CommitmentMapperScheduler {
  private redis: Redis;
  private api: BeaconApi;
  private queue: WorkQueue;
  private currentSlot: bigint;
  private lastFinalizedEpoch: bigint;
  private headSlot: bigint;
  private take: number | undefined;
  private offset: number | undefined;
  private validators: Validator[] = [];

  async init(options: any): Promise<void> {
    this.api = await getBeaconApi(options['beacon-node']);
    this.redis = new Redis(options['redis-host'], options['redis-port'], options['redis-auth']);
    this.queue = new WorkQueue(new KeyPrefix(`${CONSTANTS.validatorProofsQueue}`));
    this.take = options['take'];
    this.offset = options['offset'];
    this.headSlot = await this.api.getHeadSlot();
    this.lastFinalizedEpoch = await this.api.getLastFinalizedCheckpoint();

    const lastProcessedSlot = await this.redis.get(
      CONSTANTS.lastProcessedSlotKey,
    );

    this.currentSlot = lastProcessedSlot !== null
      ? BigInt(lastProcessedSlot)
      : (() => {
        const finalizedSlot = getLastSlotInEpoch(this.lastFinalizedEpoch);
        const slot = options['sync-slot'] || finalizedSlot;
        return BigInt(Math.min(Number(slot), Number(finalizedSlot))) - 1n;
      })();

    const lastVerifiedSlot = await this.redis.get(CONSTANTS.lastVerifiedSlotKey);

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
    this.validators = await this.redis.getValidatorsBatched(this.currentSlot);

    await this.ensureDummyProofs(this.currentSlot);

    if (runOnce) {
      this.currentSlot++;
      logProgress(this.currentSlot, this.headSlot);
      console.log(chalk.bold.blue(`Processing ${this.currentSlot} slot`));
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

  scheduleDummyProofTasks(pipeline: ChainableCommander, slot: bigint): void {
    saveDummyInput(pipeline, this.currentSlot);
    saveDummyProofPlaceholder(pipeline, 40n);
    this.pushHashValidatorProofTask(pipeline, BigInt(CONSTANTS.validatorRegistryLimit), slot);

    for (let depth = 39n; depth > 0n; depth--) {
      saveDummyProofPlaceholder(pipeline, depth);
      this.scheduleDummyProofForDepth(pipeline, depth);
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
    await this.updateValidators();

    const pipeline = this.redis.client.pipeline();
    updateLastProcessedSlot(pipeline, this.currentSlot);
    setValidatorsLengthForSlot(
      pipeline,
      this.currentSlot,
      this.validators.length,
    );
    await pipeline.exec();
  }

  async updateValidators(): Promise<void> {
    const newValidators = await this.api.getValidators(
      this.currentSlot,
      this.take,
      this.offset,
    );

    const changedValidators = newValidators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(this.validators));

    await this.modifyValidators(changedValidators, this.currentSlot);

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );
    this.validators = newValidators;
  }

  pushHashValidatorProofTask(pipeline: ChainableCommander, validatorIndex: bigint, slot: bigint): void {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.HASH_VALIDATOR_PROOF);
    dataView.setBigUint64(1, BigInt(validatorIndex), false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item)
  }

  scheduleHashValidatorProofTask(indexedValidator: IndexedValidator, slot: bigint): Promise<unknown> {
    const pipeline = this.redis.client.pipeline();

    saveRealInput(pipeline, indexedValidator, slot);
    this.pushHashValidatorProofTask(pipeline, BigInt(indexedValidator.index), slot);

    const gindex = gindexFromIndex(BigInt(indexedValidator.index), 40n);
    saveProofPlaceholder(pipeline, gindex, slot);
    recordStateModification(pipeline, `${CONSTANTS.validatorProofKey}:${gindex}`, slot);

    return pipeline.exec();
  }

  scheduleHashConcatenationTask(gindex: bigint, slot: bigint): Promise<unknown> {
    const pipeline = this.redis.client.pipeline();

    saveProofPlaceholder(pipeline, gindex, slot);
    recordStateModification(
      pipeline,
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      slot,
    );

    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.HASH_CONCATENATION_PROOF);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);

    return pipeline.exec();
  }

  async scheduleDummyProofForDepth(pipeline: ChainableCommander, depth: bigint): Promise<void> {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.DUMMY_PROOF_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);
  }

  async scheduleZeroOutValidatorTask(pipeline: ChainableCommander, validatorIndex: number, slot: bigint): Promise<void> {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.ZERO_OUT_VALIDATOR);
    dataView.setBigUint64(1, BigInt(validatorIndex), false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);
  }


  async modifyValidators(
    indexedValidators: IndexedValidator[],
    slot: bigint,
  ): Promise<void> {
    await Promise.all(indexedValidators.map((indexedValidator) => {
      return this.scheduleHashValidatorProofTask(indexedValidator, slot);
    }));

    const validatorIndices = indexedValidators.map(x => x.index);
    let levelIterator = makeBranchIterator(validatorIndices.map(BigInt), 40n);

    // ignore leaves
    levelIterator.next();

    for (const gindices of levelIterator) {
      await Promise.all(gindices.map((gindex) => this.scheduleHashConcatenationTask(gindex, slot)));
    }
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
      `${CONSTANTS.validatorProofKey}:zeroes:1`
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
      if (slot > this.currentSlot) {
        this.currentSlot = slot;
      }
    });

    eventSource.addEventListener('finalized_checkpoint', (event: any) => {
      const epoch = BigInt(JSON.parse(event.data).epoch);
      if (epoch > this.lastFinalizedEpoch) {
        this.lastFinalizedEpoch = epoch;
      }
    });
  }
}

function logProgress(currentSlot: bigint, headSlot: bigint): void {
  const progressMessage = currentSlot === headSlot
    ? chalk.cyan(currentSlot)
    : `${chalk.cyanBright(currentSlot)}/${chalk.cyan(headSlot)}`;

  console.log(chalk.bold.blue(`Syncing ${progressMessage}...`));
}

async function setValidatorsLengthForSlot(pipeline: ChainableCommander, slot: bigint, length: number): Promise<void> {
  pipeline.set(
    `${CONSTANTS.validatorsLengthKey}:${slot}`,
    length.toString(),
  );
}

function updateLastProcessedSlot(pipeline: ChainableCommander, slot: bigint): void {
  pipeline.set(CONSTANTS.lastProcessedSlotKey, slot.toString());
}

function recordStateModification(pipeline: ChainableCommander, key: string, slot: bigint): void {
  pipeline.zadd(
    `${key}:${CONSTANTS.slotLookupKey}`,
    Number(slot),
    slot.toString(),
  );
}

function saveInput(
  pipeline: ChainableCommander,
  index: bigint,
  input: CommitmentMapperInput,
  slot: bigint,
): void {
  recordStateModification(
    pipeline,
    `${CONSTANTS.validatorKey}:${index}`,
    slot,
  );

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

function saveDummyInput(
  pipeline: ChainableCommander,
  slot: bigint,
): void {
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
    }
  };

  pipeline.set(
    `${CONSTANTS.validatorProofKey}:${gindex}:${slot}`,
    JSON.stringify(obj),
  );
}

// function updateLastVerifiedSlot(pipeline: ChainableCommander, slot: bigint): void {
//   pipeline.set(CONSTANTS.lastVerifiedSlotKey, slot.toString());
// }

// Returns a function that checks whether a validator at validator index has
// changed  (doesn't check for pubkey and withdrawalCredentials since those
// never change according to the spec)
function hasValidatorChanged(prevValidators: Validator[]) {
  return ({ validator, index }: IndexedValidator): boolean =>
    prevValidators[index] === undefined ||
    validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
    validator.slashed !== prevValidators[index].slashed ||
    validator.activationEligibilityEpoch !==
    prevValidators[index].activationEligibilityEpoch ||
    validator.activationEpoch !== prevValidators[index].activationEpoch ||
    validator.exitEpoch !== prevValidators[index].exitEpoch ||
    validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch;
}

