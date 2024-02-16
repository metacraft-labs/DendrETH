import yargs from 'yargs';
import { Tree } from '@chainsafe/persistent-merkle-tree';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import Redis from 'ioredis';
import chalk from 'chalk';

import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { hexToBits } from '../../../libs/typescript/ts-utils/hex-utils';
import { computeEpochAt } from '../../../libs/typescript/ts-utils/ssz-utils';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import config from '../common_config.json';
import { Validator, ValidatorPoseidonInput } from '../../../relay/types/types';
import { convertValidatorToValidatorPoseidonInput, getZeroValidatorPoseidonInput } from './utils';

const CIRCUIT_SIZE = 8;
let TAKE: number | undefined;

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs
    .usage(
      'Usage: -redis-host <Redis host> -redis-port <Redis port> -take <number of validators>',
    )
    .option('redis-host ', {
      alias: 'redis-host',
      describe: 'The Redis host',
      type: 'string',
      default: config['redis-host'],
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: Number(config['redis-port']),
      description: 'Sets a custom redis connection',
    })
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
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
      default: undefined,
      description: 'Sets the number of validators to take',
    })
    .options('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    }).argv;

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

  const beaconApi = new BeaconApi([options['beacon-node']]);

  const slot =
    options['slot'] !== undefined
      ? options['slot']
      : Number(await beaconApi.getHeadSlot());
  const { beaconState } = await beaconApi.getBeaconState(slot);

  const offset = Number(options['offset']) || 0;
  const take = TAKE !== undefined ? TAKE + offset : undefined;
  const validators = beaconState.validators.slice(offset, take);
  beaconState.balances = beaconState.balances.slice(offset, take);
  beaconState.validators = validators;

  TAKE = validators.length;

  const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  const balancesTree = new Tree(balancesView.node);

  const balanceZeroIndex = ssz.capella.BeaconState.fields.balances.getPathInfo([
    0,
  ]).gindex;

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

  await queues[0].addItem(redis.client, new Item(buffer));

  for (let i = 0; i < 37; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(
      0,
      BigInt(validator_commitment_constants.validatorRegistryLimit),
      false,
    );

    await queues[i + 1].addItem(redis.client, new Item(buffer));
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

    await queues[0].addItem(redis.client, new Item(buffer));
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
      await queues[level].addItem(redis.client, new Item(buffer));
    }
  }

  const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

  console.log(chalk.bold.blue('Adding final proof input...'));
  await redis.saveFinalProofInput({
    stateRoot: hexToBits(
      bytesToHex(ssz.capella.BeaconState.hashTreeRoot(beaconState)),
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

  queues[38].addItem(db, new Item(new ArrayBuffer(0)));

  console.log(chalk.bold.greenBright('Done'));

  await redis.disconnect();
})();
