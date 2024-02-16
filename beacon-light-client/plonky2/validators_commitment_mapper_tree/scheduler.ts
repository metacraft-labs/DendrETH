import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { splitIntoBatches } from '../../../libs/typescript/ts-utils/common-utils';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import {
  Validator,
  IndexedValidator,
  ValidatorShaInput,
} from '../../../relay/types/types';
import chalk from 'chalk';

const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');

import CONSTANTS from '../constants/validator_commitment_constants.json';
import {
  convertValidatorToProof,
  getZeroValidatorInput,
  gindexFromIndex,
  makeBranchIterator,
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
  private currentEpoch: bigint;
  private headEpoch: bigint;
  private take: number | undefined = undefined;
  private offset: number | undefined = undefined;
  private validators: Validator[] = [];
  private ssz: any;

  async init(options: any) {
    this.redis = new Redis(options['redis-host'], options['redis-port']);
    this.take = options['take'];
    this.offset = options['offset'];
    this.queue = new WorkQueue(
      new KeyPrefix(`${CONSTANTS.validatorProofsQueue}`),
    );
    this.api = new BeaconApi([options['beacon-node']]);
    this.headEpoch = BigInt(await this.api.getHeadSlot()) / 32n;
    this.currentEpoch =
      options['sync-epoch'] !== undefined
        ? BigInt(options['sync-epoch'])
        : this.headEpoch;

    const mod = await import('@lodestar/types');
    this.ssz = mod.ssz;
  }

  async dispose() {
    return this.redis.disconnect();
  }

  async start(runOnce: boolean = false) {
    console.log(chalk.bold.blue('Fetching validators from database...'));
    this.validators = await this.redis.getValidatorsBatched(
      this.ssz,
      this.currentEpoch,
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
        `Initial syncing (${chalk.cyan(this.currentEpoch)} epoch)...`,
      ),
    );
    await this.updateValidators();

    if (runOnce) {
      return;
    }

    await this.syncEpoch();

    const es = await this.api.subscribeForEvents(['head']);
    es.on('head', async event => {
      this.headEpoch = BigInt(JSON.parse(event.data).slot) / 32n;
      await this.syncEpoch();
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
      this.currentEpoch,
    );

    await this.scheduleValidatorProof(BigInt(CONSTANTS.validatorRegistryLimit));
    await this.redis.saveZeroValidatorProof(40n);

    for (let depth = 39n; depth >= 0n; depth--) {
      this.scheduleProveZeroForDepth(depth);
      await this.redis.saveZeroValidatorProof(depth);
    }
  }

  async syncEpoch() {
    while (this.currentEpoch < this.headEpoch) {
      this.currentEpoch++;
      console.log(
        chalk.bold.blue(
          `Syncing ${
            this.currentEpoch === this.headEpoch
              ? chalk.cyan(this.currentEpoch)
              : `${chalk.cyanBright(this.currentEpoch)}/${chalk.cyan(
                  this.headEpoch,
                )}`
          }...`,
        ),
      );
      await this.updateValidators();
    }
  }

  async updateValidators() {
    const newValidators = await this.api.getValidators(
      Number(this.currentEpoch * 32n),
      this.take,
      this.offset,
    );
    const changedValidators = newValidators
      .map((validator, index) => ({ validator, index }))
      .filter(hasValidatorChanged(this.validators));

    await this.saveValidatorsInBatches(changedValidators);

    console.log(
      `Changed validators count: ${chalk.bold.yellow(
        changedValidators.length,
      )}`,
    );
    this.validators = newValidators;
  }

  async saveValidatorsInBatches(
    validators: IndexedValidator[],
    batchSize = 200,
  ) {
    for (const batch of splitIntoBatches(validators, batchSize)) {
      await this.redis.saveValidators(
        batch.map((validator: IndexedValidator) => ({
          index: validator.index,
          data: this.convertValidatorToProof(validator.validator),
        })),
        this.currentEpoch,
      );
      await Promise.all(
        batch.map(validator =>
          this.scheduleValidatorProof(BigInt(validator.index)),
        ),
      );
    }

    await this.updateBranches(validators);
  }

  async scheduleValidatorProof(validatorIndex: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, TaskTag.UPDATE_VALIDATOR_PROOF);
    dataView.setBigUint64(1, validatorIndex, false);
    dataView.setBigUint64(9, this.currentEpoch, false);
    this.queue.addItem(this.redis.client, new Item(buffer));

    // Don't create an epoch lookup for the zero validator proof
    if (validatorIndex !== BigInt(CONSTANTS.validatorRegistryLimit)) {
      await this.redis.addToEpochLookup(
        `${CONSTANTS.validatorProofKey}:${gindexFromIndex(
          validatorIndex,
          40n,
        )}`,
        this.currentEpoch,
      );
    }
  }

  async scheduleUpdateProofNodeTask(gindex: bigint) {
    const buffer = new ArrayBuffer(17);
    const dataView = new DataView(buffer);

    await this.redis.addToEpochLookup(
      `${CONSTANTS.validatorProofKey}:${gindex}`,
      this.currentEpoch,
    );

    dataView.setUint8(0, TaskTag.UPDATE_PROOF_NODE);
    dataView.setBigUint64(1, gindex, false);
    dataView.setBigUint64(9, this.currentEpoch, false);
    this.queue.addItem(this.redis.client, new Item(buffer));
  }

  async updateBranches(validators: IndexedValidator[]) {
    let levelIterator = makeBranchIterator(
      validators.map(validator => BigInt(validator.index)),
      40n,
    );

    let leafs = levelIterator.next().value!;

    await Promise.all(
      leafs.map(gindex =>
        this.redis.saveValidatorProof(gindex, this.currentEpoch),
      ),
    );

    for (const gindices of levelIterator) {
      await Promise.all(
        gindices.map(gindex =>
          this.redis.saveValidatorProof(gindex, this.currentEpoch),
        ),
      );

      await Promise.all(
        gindices.map(gindex => this.scheduleUpdateProofNodeTask(gindex)),
      );
    }
  }

  async scheduleProveZeroForDepth(depth: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, TaskTag.PROVE_ZERO_FOR_DEPTH);
    dataView.setBigUint64(1, depth, false);

    this.queue.addItem(this.redis.client, new Item(buffer));
  }

  convertValidatorToProof(validator: Validator): ValidatorShaInput {
    return convertValidatorToProof(validator, this.ssz);
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
