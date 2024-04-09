import { splitIntoBatches } from '@dendreth/utils/ts-utils/common-utils';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { Validator, IndexedValidator } from '@dendreth/relay/types/types';
import chalk from 'chalk';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../constants/validator_commitment_constants.json';
import {
  convertValidatorToProof,
  getZeroValidatorInput,
  gindexFromIndex,
  makeBranchIterator,
  getLastSlotInEpoch,
} from './utils';

enum TaskTag {
  UPDATE_PROOF_NODE = 0,
  PROVE_ZERO_FOR_DEPTH = 1,
  UPDATE_VALIDATOR_PROOF = 2,
}

export class CommitmentMapperScheduler {
  private redis: Redis;
  private api: BeaconApi;
  private queue: any;
  private currentSlot: bigint;
  private lastFinalizedEpoch: bigint;
  private headSlot: bigint;
  private take: number | undefined = undefined;
  private offset: number | undefined = undefined;
  private validators: Validator[] = [];
  private ssz: any;
  // This is used to prevent spawning multiple async jobs on head event
  private isSyncing = false;
  // This is used to allow rerunning the script if it dies or is intentionally
  // stopped
  private isFirstTimeRunning: boolean;

  async init(options: any) {
    this.api = await getBeaconApi(options['beacon-node']);
    this.redis = new Redis(options['redis-host'], options['redis-port']);
    this.queue = new WorkQueue(
      new KeyPrefix(`${CONSTANTS.validatorProofsQueue}`),
    );

    this.take = options['take'];
    this.offset = options['offset'];
    this.headSlot = await this.api.getHeadSlot();

    this.lastFinalizedEpoch = await this.api.getLastFinalizedCheckpoint();
    const lastVerifiedSlot = await this.redis.get(CONSTANTS.lastVerifiedSlotKey);
    if (lastVerifiedSlot === null) {
      this.isFirstTimeRunning = true;
      await this.redis.set(
        CONSTANTS.lastVerifiedSlotKey,
        `${getLastSlotInEpoch(this.lastFinalizedEpoch)}`,
      );
    } else {
      this.isFirstTimeRunning = false;
    }

    const lastProcessedSlot = await this.redis.get(CONSTANTS.lastProcessedSlotKey);
    if (lastProcessedSlot === null) {
      this.currentSlot = (() => {
        const firstNonFinalizedSlot = this.lastFinalizedEpoch * 32n + 32n;
        const slot = options['sync-slot'] || firstNonFinalizedSlot;
        return BigInt(Math.min(Number(slot), Number(firstNonFinalizedSlot)));
      })();
    } else {
      this.currentSlot = BigInt(lastProcessedSlot) + 1n;
    }

    const mod = await import('@lodestar/types');
    this.ssz = mod.ssz;
  }

  async dispose() {
    return this.redis.quit();
  }

  async start(runOnce: boolean = false) {
    console.log(chalk.bold.blue('Fetching validators from database...'));
    this.validators = await this.redis.getValidatorsBatched(
      this.ssz,
      this.currentSlot,
    );
    console.log(
      `Loaded ${chalk.bold.yellow(
        this.validators.length,
      )} validators from database`,
    );

    if (await this.redis.isZeroValidatorEmpty()) {
      console.log(chalk.bold.blue('Adding zero tasks...'));
      await this.scheduleZeroTasks();
    }

    console.log(
      chalk.bold.blue(
        `Initial syncing (${chalk.cyan(this.currentSlot)} slot)...`,
      ),
    );
    await this.updateValidators();

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

  async scheduleZeroTasks() {
    await this.redis.saveValidators(
      [
        {
          index: Number(CONSTANTS.validatorRegistryLimit),
          data: getZeroValidatorInput(),
        },
      ],
      this.currentSlot,
    );

    await this.scheduleValidatorProof(
      BigInt(CONSTANTS.validatorRegistryLimit),
      this.currentSlot,
    );
    await this.redis.saveZeroValidatorProof(40n);

    for (let depth = 39n; depth >= 0n; depth--) {
      this.scheduleProveZeroForDepth(depth);
      await this.redis.saveZeroValidatorProof(depth);
    }
  }

  async syncToHeadSlot(isInitialSyncing: boolean) {
    this.isSyncing = true;

    while (this.currentSlot < this.headSlot) {
      this.currentSlot++;

      console.log(
        chalk.bold.blue(
          `Syncing ${this.currentSlot === this.headSlot
            ? chalk.cyan(this.currentSlot)
            : `${chalk.cyanBright(this.currentSlot)}/${chalk.cyan(
              this.headSlot,
            )}`
          }...`,
        ),
      );

      await this.redis.updateLastProcessedSlot(this.currentSlot);
      if (isInitialSyncing && this.currentSlot <= getLastSlotInEpoch(this.lastFinalizedEpoch)) {
        this.redis.updateLastVerifiedSlot(this.currentSlot);
      }
      await this.updateValidators();
    }

    this.isSyncing = false;
  }

  async updateValidators() {
    const newValidators = await this.api.getValidators(
      this.currentSlot,
      this.take,
      this.offset,
    );

    const changedValidators = newValidators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(this.validators));

    await this.redis.setValidatorsLength(
      this.currentSlot,
      newValidators.length,
    );
    await this.saveValidatorsInBatches(changedValidators);

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );
    this.validators = newValidators;
  }

  public async saveValidatorsInBatches(
    validators: IndexedValidator[],
    slot = this.currentSlot,
    batchSize = 200,
  ) {
    for (const batch of splitIntoBatches(validators, batchSize)) {
      await this.redis.saveValidators(
        batch.map((validator: IndexedValidator) => ({
          index: validator.index,
          data: convertValidatorToProof(validator.validator, this.ssz),
        })),
        slot,
      );
      await Promise.all(
        batch.map(validator =>
          this.scheduleValidatorProof(BigInt(validator.index), slot),
        ),
      );
    }

    await this.updateBranches(validators, slot);
  }

  async scheduleValidatorProof(validatorIndex: bigint, slot: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    dataView.setBigUint64(9, this.currentSlot, false);
    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));

    // Don't create an slot lookup for the zero validator proof
    if (validatorIndex !== BigInt(CONSTANTS.validatorRegistryLimit)) {
      await this.redis.addToSlotLookup(
        `${CONSTANTS.validatorProofKey}:${gindexFromIndex(
          validatorIndex,
          40n,
        )}`,
        slot,
      );
    }
  }

  async scheduleUpdateProofNodeTask(gindex: bigint, slot: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    await this.redis.addToSlotLookup(
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      slot,
    );

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, slot, false);
    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));
  }

  async updateBranches(validators: IndexedValidator[], slot: bigint) {
    let levelIterator = makeBranchIterator(
      validators.map(validator => BigInt(validator.index)),
      40n,
    );

    let leafs = levelIterator.next().value!;

    await Promise.all(
      leafs.map(gindex => this.redis.saveValidatorProof(gindex, slot)),
    );

    for (const gindices of levelIterator) {
      await Promise.all(
        gindices.map(gindex => this.redis.saveValidatorProof(gindex, slot)),
      );

      await Promise.all(
        gindices.map(gindex => this.scheduleUpdateProofNodeTask(gindex, slot)),
      );
    }
  }

  async scheduleProveZeroForDepth(depth: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.PROVE_ZERO_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));
  }
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
