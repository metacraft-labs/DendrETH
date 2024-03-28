import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import Redis from 'ioredis';
import { getBeaconApi } from '../../../relay/implementations/beacon-api';
import { bigint_to_array } from '../../solidity/test/utils/bls';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import * as fs from 'fs';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { computeEpochAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
const CIRCUIT_SIZE = 8;
let TAKE;

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
      default: '127.0.0.1',
      description: 'Sets a custom redis connection',
    })
    .option('redis-port', {
      alias: 'redis-port',
      describe: 'The Redis port',
      type: 'number',
      default: 6379,
      description: 'Sets a custom redis connection',
    })
    .option('beacon-node', {
      alias: 'beacon-node',
      describe: 'The beacon node url',
      type: 'string',
      default: 'http://unstable.mainnet.beacon-api.nimbus.team',
      description: 'Sets a custom beacon node url',
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
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];
  let MOCK = options['mock'];
  let GRANULITY = MOCK ? 1000 : 1;

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
    : await beaconApi.getBeaconState(6953401);

  const validators = beaconState.validators.slice(0, TAKE);
  TAKE = validators.length;

  beaconState.validators = validators;
  beaconState.balances = beaconState.balances.slice(0, TAKE);

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

  console.log('Adding tasks about zeros');
  await redis.saveValidatorBalancesInput([
    {
      index: Number(validator_commitment_constants.validatorRegistryLimit),
      input: JSON.stringify({
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
      }),
    },
  ]);

  const buffer = new ArrayBuffer(8);
  const dataView = new DataView(buffer);

  dataView.setBigUint64(
    0,
    BigInt(validator_commitment_constants.validatorRegistryLimit),
    false,
  );

  await queues[0].addItem(db, new Item(Buffer.from(buffer)));

  for (let i = 0; i < 38; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, BigInt(i), false);
    dataView.setBigUint64(
      8,
      BigInt(validator_commitment_constants.validatorRegistryLimit),
      false,
    );
    dataView.setBigUint64(
      16,
      BigInt(validator_commitment_constants.validatorRegistryLimit),
      false,
    );

    await queues[i + 1].addItem(db, new Item(Buffer.from(buffer)));

    if (i % (GRANULITY / 10) === 0 && i !== 0) console.log('Added zeros tasks');
  }

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
        index: j * CIRCUIT_SIZE,
        input: JSON.stringify({
          balances: balances.slice(
            j * (CIRCUIT_SIZE / 4),
            (j + 1) * (CIRCUIT_SIZE / 4),
          ),
          validators: [
            ...validators
              .slice(j * CIRCUIT_SIZE, (j + 1) * CIRCUIT_SIZE)
              .map(v => convertValidator(v)),
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
        }),
      });
    }

    await redis.saveValidatorBalancesInput(batch);

    if (i % GRANULITY === 0 && i !== 0) console.log('saved batch', i);
  }

  for (let i = 0; i < TAKE / CIRCUIT_SIZE; i++) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i * CIRCUIT_SIZE), false);

    await queues[0].addItem(db, new Item(Buffer.from(buffer)));
    if (i % (GRANULITY * 100) === 0 && i !== 0)
      console.log(`added ${i * CIRCUIT_SIZE}`);
  }

  for (let j = 1; j < 38; j++) {
    console.log('Added inner level of proofs', j);

    let prev_index = 2199023255552n;

    for (let i = 0; i < TAKE / CIRCUIT_SIZE; i++) {
      const buffer = new ArrayBuffer(24);
      const view = new DataView(buffer);

      let index = BigInt(i * CIRCUIT_SIZE);

      if (
        index / 2n ** (BigInt(j) + 3n) ==
        prev_index / 2n ** (BigInt(j) + 3n)
      ) {
        continue;
      }

      const { first, second } = calculateIndexes(
        BigInt(i * CIRCUIT_SIZE),
        BigInt(j),
      );
      if (i % (GRANULITY * 100) === 0 && i !== 0)
        console.log(`added ${j}:${first}:${second}`);

      view.setBigUint64(0, BigInt(j - 1), false);
      view.setBigUint64(8, first, false);
      view.setBigUint64(16, second, false);

      await redis.saveBalanceProof(BigInt(j - 1), first);
      await queues[j].addItem(db, new Item(Buffer.from(buffer)));

      prev_index = first;
    }
  }

  const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

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

  queues[39].addItem(db, new Item(Buffer.from(new ArrayBuffer(0))));

  console.log('Added final proof input');

  console.log('ready');

  process.exit(0);
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

function convertValidator(validator): any {
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

function calculateIndexes(index: bigint, depth: bigint) {
  let first: bigint = index;
  let second: bigint = index + 8n;

  for (let k = 3n; k < depth + 3n; k++) {
    if (first % 2n ** (k + 1n) == 0n) {
      second = first + 2n ** k;
    } else {
      second = first;
      first = first - 2n ** k;
    }
  }

  return { first, second };
}
