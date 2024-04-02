import chalk from 'chalk';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { computeEpochAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import { CommandLineOptionsBuilder } from '../cmdline';
import config from '../common_config.json';
import {
  convertValidatorToValidatorPoseidonInput,
  getZeroValidatorPoseidonInput,
} from './utils';

const CIRCUIT_SIZE = 8;
let TAKE: number;

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = new CommandLineOptionsBuilder()
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .withRedisOpts()
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'array',
      default: config['beacon-node'],
      description: 'Sets a custom beacon node url',
    })
    .option('slot', {
      alias: 'slot',
      describe: 'The state slot',
      type: 'number',
      default: undefined,
      description: 'Fetches the balances for this slot',
    })
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: Infinity,
      description: 'Sets the number of validators to take',
    })
    .option('mock', {
      alias: 'mock',
      describe: 'Runs the tool without doing actual calculations',
      type: 'boolean',
      default: false,
      description: 'Runs the tool without doing actual calculations.',
    })
    .option('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    })
    .build();

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  TAKE = options['take'];

  const queues: any[] = [];

  for (let i = 0; i < 38; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${validator_commitment_constants.balanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }

  queues.push(
    new WorkQueue(
      new KeyPrefix(
        `${validator_commitment_constants.balanceVerificationQueue}:final`,
      ),
    ),
  );

  const beaconApi = await getBeaconApi(options['beacon-node']);

  const slot =
    options['slot'] !== undefined
      ? BigInt(options['slot'])
      : await beaconApi.getHeadSlot();
  const { beaconState } =
    (await beaconApi.getBeaconState(slot)) ||
    panic('Could not fetch beacon state');

  const currentSSZFork = await beaconApi.getCurrentSSZ(slot);

  const offset = Number(options['offset']) || 0;
  const take = TAKE !== Infinity ? TAKE + offset : Infinity;
  const validators = beaconState.validators.slice(offset, take);
  beaconState.balances = beaconState.balances.slice(offset, take);
  beaconState.validators = validators;

  TAKE = validators.length;

  const balancesView = currentSSZFork.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const balancesTree = new Tree(balancesView.node);

  const balanceZeroIndex =
    currentSSZFork.BeaconState.fields.balances.getPathInfo([0]).gindex;

  const balances: number[][] = [];

  for (let i = 0; i < TAKE / 4; i++) {
    balances.push(
      hexToBits(
        bytesToHex(balancesTree.getNode(balanceZeroIndex + BigInt(i)).root),
      ),
    );
  }

  if (balances.length % (CIRCUIT_SIZE / 4) !== 0) {
    balances.push(''.padStart(256, '0').split('').map(Number));
  }

  await redis.saveValidatorBalancesInput([
    {
      index: Number(validator_commitment_constants.validatorRegistryLimit),
      input: {
        balances: Array(CIRCUIT_SIZE / 4)
          .fill('')
          .map(() => ''.padStart(256, '0').split('').map(Number)),
        validators: Array(CIRCUIT_SIZE).fill(getZeroValidatorPoseidonInput()),
        withdrawalCredentials: [
          hexToBits(
            '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
          ),
        ],
        currentEpoch: computeEpochAt(beaconState.slot).toString(),
        validatorIsZero: Array(CIRCUIT_SIZE).fill(1),
      },
    },
  ]);

  console.log(chalk.bold.blue('Adding zero tasks...'));

  const buffer = new ArrayBuffer(8);
  const dataView = new DataView(buffer);

  dataView.setBigUint64(
    0,
    BigInt(validator_commitment_constants.validatorRegistryLimit),
    false,
  );

  await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));

  for (let i = 0; i < 37; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(
      0,
      BigInt(validator_commitment_constants.validatorRegistryLimit),
      false,
    );

    await queues[i + 1].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  console.log(chalk.bold.blue('Saving validator balance input...'));
  const batchSize = 100;
  for (let i = 0; i <= TAKE / CIRCUIT_SIZE / batchSize; i++) {
    let batch: any[] = [];
    for (
      let j = i * batchSize;
      j < i * batchSize + batchSize && j < TAKE / CIRCUIT_SIZE;
      j++
    ) {
      let size =
        (j + 1) * CIRCUIT_SIZE <= validators.length
          ? CIRCUIT_SIZE
          : validators.length - j * CIRCUIT_SIZE;

      let array = new Array(size).fill(0);

      batch.push({
        index: j,
        input: {
          balances: balances.slice(
            j * (CIRCUIT_SIZE / 4),
            (j + 1) * (CIRCUIT_SIZE / 4),
          ),
          validators: [
            ...validators
              .slice(j * CIRCUIT_SIZE, (j + 1) * CIRCUIT_SIZE)
              .map(v => convertValidatorToValidatorPoseidonInput(v)),
            ...Array(
              (j + 1) * CIRCUIT_SIZE -
                Math.min((j + 1) * CIRCUIT_SIZE, validators.length),
            ).fill(getZeroValidatorPoseidonInput()),
          ],
          withdrawalCredentials: [
            hexToBits(
              '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
            ),
          ],
          currentEpoch: computeEpochAt(beaconState.slot).toString(),
          validatorIsZero: array.concat(new Array(CIRCUIT_SIZE - size).fill(1)),
        },
      });
    }

    await redis.saveValidatorBalancesInput(batch);
  }

  await redis.saveBalanceProof(
    0n,
    BigInt(validator_commitment_constants.validatorRegistryLimit),
  );

  for (let i = 0; i < TAKE / CIRCUIT_SIZE; i++) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i), false);

    await redis.saveBalanceProof(0n, BigInt(i));

    await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level < 38; level++) {
    await redis.saveBalanceProof(
      BigInt(level),
      BigInt(validator_commitment_constants.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(TAKE / CIRCUIT_SIZE / 2 ** level)).keys(),
    ];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveBalanceProof(BigInt(level), BigInt(key));

      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(Buffer.from(buffer)));
    }
  }

  const beaconStateView = currentSSZFork.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

  console.log(chalk.bold.blue('Adding final proof input...'));
  await redis.saveFinalProofInput({
    stateRoot: hexToBits(
      bytesToHex(currentSSZFork.BeaconState.hashTreeRoot(beaconState)),
    ),
    slot: beaconState.slot.toString(),
    slotBranch: beaconStateTree
      .getSingleProof(34n)
      .map(x => hexToBits(bytesToHex(x))),
    withdrawalCredentials: [
      hexToBits(
        '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
      ),
    ],
    balanceBranch: beaconStateTree
      .getSingleProof(44n)
      .map(x => hexToBits(bytesToHex(x))),
    validatorsBranch: beaconStateTree
      .getSingleProof(43n)
      .map(x => hexToBits(bytesToHex(x))),
    validatorsSizeBits: hexToBits(bytesToHex(ssz.UintNum64.hashTreeRoot(TAKE))),
  });

  // NOTE: Maybe this is unecessarry
  queues[38].addItem(redis.client, new Item(Buffer.from(new ArrayBuffer(0))));

  console.log(chalk.bold.greenBright('Done'));

  await redis.quit();
})();
