import {
  getFirstSlotInEpoch,
  getLastSlotInEpoch,
  gindexFromIndex,
  indexFromGindex,
  makeBranchIterator,
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
  getDummyCommitmentMapperInput,
} from '../../utils/common_utils';
import { ChainableCommander } from 'ioredis';

enum TaskTag {
  CONCATENATE_HASHES = 0,
  PROVE_DUMMY_FOR_DEPTH = 1,
  VALIDATOR_HASH_PROOF = 2,
  ZERO_OUT_VALIDATOR = 3,
}

export class CommitmentMapperScheduler {
  private redis: Redis;
  private api: BeaconApi;
  private queue: WorkQueue;
  private currentSlot: bigint;
  private lastFinalizedEpoch: bigint;
  private headSlot: bigint;
  private take: number | undefined = undefined;
  private offset: number | undefined = undefined;
  private validators: Validator[] = [];
  // This is used to prevent spawning multiple async jobs on head event
  private isSyncing = false;
  // This is used to allow rerunning the script if it dies or is intentionally
  // stopped
  private isFirstTimeRunning: boolean;

  async init(options: any) {
    this.api = await getBeaconApi(options['beacon-node']);
    this.redis = new Redis(options['redis-host'], options['redis-port'], options['redis-auth']);
    this.queue = new WorkQueue(
      new KeyPrefix(`${CONSTANTS.validatorProofsQueue}`),
    );

    this.take = options['take'];
    this.offset = options['offset'];
    this.headSlot = await this.api.getHeadSlot();

    this.lastFinalizedEpoch = await this.api.getLastFinalizedCheckpoint();
    const lastVerifiedSlot = await this.redis.get(
      CONSTANTS.lastVerifiedSlotKey,
    );
    if (lastVerifiedSlot === null) {
      this.isFirstTimeRunning = true;
      await this.redis.set(
        CONSTANTS.lastVerifiedSlotKey,
        `${getLastSlotInEpoch(this.lastFinalizedEpoch)}`,
      );
    } else {
      this.isFirstTimeRunning = false;
    }

    const lastProcessedSlot = await this.redis.get(
      CONSTANTS.lastProcessedSlotKey,
    );
    if (lastProcessedSlot === null) {
      this.currentSlot = (() => {
        const firstNonFinalizedSlot = getFirstSlotInEpoch(
          this.lastFinalizedEpoch + 1n,
        );
        const slot = options['sync-slot'] || firstNonFinalizedSlot;
        return BigInt(Math.min(Number(slot), Number(firstNonFinalizedSlot)));
      })();
    } else {
      this.currentSlot = BigInt(lastProcessedSlot) + 1n;
    }
  }

  async dispose() {
    return this.redis.quit();
  }

  async start(runOnce: boolean = false) {
    console.log(chalk.bold.blue('Fetching validators from database...'));
    this.validators = await this.redis.getValidatorsBatched(this.currentSlot);

    await this.ensureZeroes();

    // TODO: Rewrite log. Initial syncing must be logged just the first time the
    // script is ran
    console.log(
      chalk.bold.blue(
        `Initial syncing (${chalk.cyan(this.currentSlot)} slot)...`,
      ),
    );

    const pipeline = this.redis.client.pipeline();
    await this.updateValidators(pipeline);
    await pipeline.exec();

    if (runOnce) {
      return;
    }

    await this.syncToHeadSlot(this.isFirstTimeRunning);

    const eventSource = this.api.subscribeForEvents([
      'head',
      'finalized_checkpoint',
    ]);
    eventSource.addEventListener('head', async (event: any) => {
      this.headSlot = BigInt(JSON.parse(event.data).slot);
      // Guarding against api taking too long to respond and firing two async
      // calls to this function simultaneously
      if (!this.isSyncing) {
        await this.syncToHeadSlot(false);
      }
    });
    eventSource.addEventListener('finalized_checkpoint', (event: any) => {
      this.lastFinalizedEpoch = BigInt(JSON.parse(event.data).epoch);
    });
  }

  async scheduleDummyProofTasks(pipeline: ChainableCommander) {
    this.saveDummyInput(pipeline, this.currentSlot);

    this.scheduleHashValidatorTask(
      pipeline,
      BigInt(CONSTANTS.validatorRegistryLimit),
      this.currentSlot,
    );
    await this.redis.saveDummyValidatorProof(40n);

    for (let depth = 39n; depth >= 0n; depth--) {
      this.scheduleProveDummyForDepth(pipeline, depth);
      await this.redis.saveDummyValidatorProof(depth);
    }
  }

  async syncToHeadSlot(isInitialSyncing: boolean) {
    this.isSyncing = true;

    while (this.currentSlot < this.headSlot) {
      this.currentSlot++;

      const progressMessage = this.currentSlot === this.headSlot
        ? chalk.cyan(this.currentSlot)
        : `${chalk.cyanBright(this.currentSlot)}/${chalk.cyan(this.headSlot)}`;

      console.log(chalk.bold.blue(`Syncing ${progressMessage}...`));

      const pipeline = this.redis.client.pipeline();

      this.updateLastProcessedSlot(pipeline, this.currentSlot);
      if (
        isInitialSyncing &&
        this.currentSlot <= getLastSlotInEpoch(this.lastFinalizedEpoch)
      ) {
        this.updateLastVerifiedSlot(pipeline, this.currentSlot);
      }
      this.updateValidators(pipeline);

      await pipeline.exec();
    }

    this.isSyncing = false;
  }

  async updateValidators(pipeline: ChainableCommander) {
    const newValidators = await this.api.getValidators(
      this.currentSlot,
      this.take,
      this.offset,
    );

    const changedValidators = newValidators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(this.validators));

    this.modifyValidators(pipeline, changedValidators, this.currentSlot);

    setValidatorsLength(
      pipeline,
      this.currentSlot,
      newValidators.length,
    );

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );
    this.validators = newValidators;
  }

  scheduleHashValidatorTask(pipeline: ChainableCommander, validatorIndex: bigint, slot: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.VALIDATOR_HASH_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item)

    // Don't create a slot lookup for the zero validator proof
    if (validatorIndex !== BigInt(CONSTANTS.validatorRegistryLimit)) {
      this.addToSlotLookup(
        pipeline,
        `${CONSTANTS.validatorProofKey}:${gindexFromIndex(
          validatorIndex,
          40n,
        )}`,
        slot,
      );
    }
  }

  async scheduleConcatenateHashesTask(pipeline: ChainableCommander, gindex: bigint, slot: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    this.addToSlotLookup(
      pipeline,
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      slot,
    );

    dataView.setUint8(0, TaskTag.CONCATENATE_HASHES);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);
  }

  async scheduleProveDummyForDepth(pipeline: ChainableCommander, depth: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.PROVE_DUMMY_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);
  }

  async scheduleZeroOutValidatorTask(pipeline: ChainableCommander, validatorIndex: number, slot: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.ZERO_OUT_VALIDATOR);
    dataView.setBigUint64(1, BigInt(validatorIndex), false);
    dataView.setBigUint64(9, slot, false);

    const item = new Item(Buffer.from(buffer));
    this.queue.addItemToPipeline(pipeline, item);
  }

  addToSlotLookup(pipeline: ChainableCommander, key: string, slot: bigint) {
    pipeline.zadd(
      `${key}:${CONSTANTS.slotLookupKey}`,
      Number(slot),
      slot.toString(),
    );
  }

  removeFromSlotLookup(pipeline: ChainableCommander, key: string, ...slots: bigint[]) {
    pipeline.zrem(
      `${key}:${CONSTANTS.slotLookupKey}`,
      slots.map(String),
    );
  }

  //   updateValidators() {
  //     pipeline: ChainableCommander
  //     indexedValidators: IndexedValidator[],
  //       slot: bigint,
  //   } {
  //
  // }

  saveInput(
    pipeline: ChainableCommander,
    index: bigint,
    input: CommitmentMapperInput,
    slot: bigint,
  ) {
    this.addToSlotLookup(
      pipeline,
      `${CONSTANTS.validatorKey}:${index}`,
      slot,
    );

    pipeline.set(
      `${CONSTANTS.validatorKey}:${index}:${slot}`,
      JSON.stringify(input),
    );
  }

  saveRealInput(
    pipeline: ChainableCommander,
    { validator, index }: IndexedValidator,
    slot: bigint,
  ) {
    const input = commitmentMapperInputFromValidator(validator);
    this.saveInput(pipeline, BigInt(index), input, slot);
  }

  saveDummyInput(
    pipeline: ChainableCommander,
    slot: bigint,
  ) {
    const index = BigInt(CONSTANTS.validatorRegistryLimit);
    const input = getDummyCommitmentMapperInput();
    this.saveInput(pipeline, index, input, slot);
  }

  modifyValidators(
    pipeline: ChainableCommander,
    indexedValidators: IndexedValidator[],
    slot: bigint,
  ) {
    indexedValidators.forEach(indexedValidator => {
      this.saveRealInput(pipeline, indexedValidator, slot);
    });

    const validatorIndices = indexedValidators.map(x => x.index);
    let levelIterator = makeBranchIterator(validatorIndices.map(BigInt), 40n);

    let leaves = levelIterator.next().value!;
    leaves.forEach(gindex => this.saveValidatorProof(pipeline, gindex, slot));
    leaves.forEach(gindex => this.scheduleHashValidatorTask(pipeline, indexFromGindex(gindex, 40n), slot));

    for (const gindices of levelIterator) {
      gindices.forEach(gindex => this.saveValidatorProof(pipeline, gindex, slot));
      gindices.forEach(gindex => this.scheduleConcatenateHashesTask(pipeline, gindex, slot));
    }
  }

  saveValidatorProof(
    pipeline: ChainableCommander,
    gindex: bigint,
    slot: bigint,
    proof: ValidatorProof = {
      needsChange: true,
      proofKey: '',
      publicInputs: {
        poseidonHashTreeRoot: [0, 0, 0, 0],
        sha256HashTreeRoot: ''.padEnd(64, '0'),
      },
    },
  ) {
    pipeline.set(
      `${CONSTANTS.validatorProofKey}:${gindex}:${slot}`,
      JSON.stringify(proof),
    );
  }

  updateLastProcessedSlot(pipeline: ChainableCommander, slot: bigint) {
    pipeline.set(CONSTANTS.lastProcessedSlotKey, slot.toString());
  }

  updateLastVerifiedSlot(pipeline: ChainableCommander, slot: bigint) {
    pipeline.set(CONSTANTS.lastVerifiedSlotKey, slot.toString());
  }

  async ensureZeroes() {
    const pipeline = this.redis.client.pipeline();

    if (await this.isInitialization()) {
      console.log(chalk.bold.blue('Adding zero tasks...'));
      this.scheduleDummyProofTasks(pipeline);
    }
    await pipeline.exec();
  }

  async isInitialization() {
    const result = await this.redis.client.exists(
      `${CONSTANTS.lastProcessedSlotKey}`
    );

    return result === 0;
  }
}

async function setValidatorsLength(pipeline: ChainableCommander, slot: bigint, length: number) {
  pipeline.set(
    `${CONSTANTS.validatorsLengthKey}:${slot}`,
    length.toString(),
  );
}

// Returns a function that checks whether a validator at validator index has
// changed  (doesn't check for pubkey and withdrawalCredentials since those
// never change according to the spec)
function hasValidatorChanged(prevValidators: Validator[]) {
  return ({ validator, index }: IndexedValidator) =>
    prevValidators[index] === undefined ||
    validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
    validator.slashed !== prevValidators[index].slashed ||
    validator.activationEligibilityEpoch !==
    prevValidators[index].activationEligibilityEpoch ||
    validator.activationEpoch !== prevValidators[index].activationEpoch ||
    validator.exitEpoch !== prevValidators[index].exitEpoch ||
    validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch;
}

