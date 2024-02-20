import yargs from 'yargs';
import * as fs from 'fs';
import chalk from 'chalk';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import { hideBin } from 'yargs/helpers';
import { computeEpochAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import config from '../common_config.json';
import { Validator } from '@dendreth/relay/types/types';

const CIRCUIT_SIZE = 8;
let TAKE: number;

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs(hideBin(process.argv))
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
    .options('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: undefined,
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  TAKE = options['take'];
  let MOCK = options['mock'];

  const queues: any[] = [];

  for (let i = 0; i < 39; i++) {
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

  const beaconApi = await getBeaconApi([options['beacon-node']]);

  const beaconState_bin = fs.existsSync('../mock_data/beaconState.bin')
    ? '../mock_data/beaconState.bin'
    : 'mock_data/beaconState.bin';

  const { beaconState } = MOCK
    ? {
        beaconState: ssz.capella.BeaconState.deserialize(
          fs.readFileSync(beaconState_bin),
        ),
      }
    : (await beaconApi.getBeaconState(
        options['slot'] !== undefined
          ? options['slot']
          : Number(await beaconApi.getHeadSlot()),
      )) || panic('Could not fetch beacon state');

  const offset = Number(options['offset']) || 0;
  const take = TAKE !== Infinity ? TAKE + offset : Infinity;
  const validators = beaconState.validators.slice(offset, take);
  beaconState.balances = beaconState.balances.slice(offset, take);
  beaconState.validators = validators;

  TAKE = validators.length;

  const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
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
        validators: Array(CIRCUIT_SIZE).fill(getZeroValidator()),
        withdrawalCredentials: computeNumberFromLittleEndianBits(
          hexToBits(
            '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
          ),
        ).toString(),
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

  for (let i = 0; i < 38; i++) {
    const buffer = new ArrayBuffer(8);
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
              .map((v: Validator) => convertValidator(v)),
            ...Array(
              (j + 1) * CIRCUIT_SIZE -
                Math.min((j + 1) * CIRCUIT_SIZE, validators.length),
            ).fill(getZeroValidator()),
          ],
          withdrawalCredentials: computeNumberFromLittleEndianBits(
            hexToBits(
              '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
            ),
          ).toString(),
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
    withdrawalCredentials: computeNumberFromLittleEndianBits(
      hexToBits(
        '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
      ),
    ).toString(),
    balanceBranch: beaconStateTree
      .getSingleProof(44n)
      .map(x => hexToBits(bytesToHex(x))),
    validatorsBranch: beaconStateTree
      .getSingleProof(43n)
      .map(x => hexToBits(bytesToHex(x))),
    validatorsSizeBits: hexToBits(bytesToHex(ssz.UintNum64.hashTreeRoot(TAKE))),
  });

  queues[39].addItem(redis.client, new Item(Buffer.from(new ArrayBuffer(0))));

  console.log(chalk.bold.greenBright('Done'));

  await redis.disconnect();
})();

function getZeroValidator() {
  return {
    pubkey: '0',
    withdrawalCredentials: '0',
    effectiveBalance: '0',
    slashed: 0,
    activationEligibilityEpoch: '0',
    activationEpoch: '0',
    exitEpoch: '0',
    withdrawableEpoch: '0',
  };
}

function convertValidator(validator: Validator): any {
  return {
    pubkey: computeNumberFromLittleEndianBits(
      hexToBits(bytesToHex(validator.pubkey), 384),
    ).toString(),
    withdrawalCredentials: computeNumberFromLittleEndianBits(
      hexToBits(bytesToHex(validator.withdrawalCredentials)),
    ).toString(),
    effectiveBalance: validator.effectiveBalance.toString(),
    slashed: Number(validator.slashed),
    activationEligibilityEpoch: validator.activationEligibilityEpoch.toString(),
    activationEpoch: validator.activationEpoch.toString(),
    exitEpoch:
      validator.exitEpoch === Infinity
        ? (2n ** 64n - 1n).toString()
        : validator.exitEpoch.toString(),
    withdrawableEpoch:
      validator.withdrawableEpoch === Infinity
        ? (2n ** 64n - 1n).toString()
        : validator.withdrawableEpoch.toString(),
  };
}

function computeNumberFromLittleEndianBits(bits: number[]) {
  return BigInt('0b' + bits.join(''));
}
