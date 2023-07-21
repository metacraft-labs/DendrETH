import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import Redis from 'ioredis';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { hexToBits } from '../../../libs/typescript/ts-utils/hex-utils';
import { bigint_to_array } from '../../solidity/test/utils/bls';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import validator_commitment_constants from '../constants/validator_commitment_constants.json';
import yargs from 'yargs';

const CIRCUIT_SIZE = 8;
let TAKE;

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
    .option('take', {
      alias: 'take',
      describe: 'The number of validators to take',
      type: 'number',
      default: 1024,
      description: 'Sets the number of validators to take',
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);

  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];

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

  const beaconApi = new BeaconApi([
    'http://unstable.mainnet.beacon-api.nimbus.team',
  ]);

  const { beaconState } = await beaconApi.getBeaconState(6953401);

  const validators = beaconState.validators.slice(0, TAKE);

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

  const batchSize = 100;
  for (let i = 0; i <= TAKE / CIRCUIT_SIZE / batchSize; i++) {
    let batch: any[] = [];
    for (
      let j = i * batchSize;
      j < i * batchSize + batchSize && j < TAKE / CIRCUIT_SIZE;
      j++
    ) {
      batch.push({
        index: j * CIRCUIT_SIZE,
        input: JSON.stringify({
          balances: balances.slice(
            j * (CIRCUIT_SIZE / 4),
            (j + 1) * (CIRCUIT_SIZE / 4),
          ),
          validators: validators
            .slice(j * CIRCUIT_SIZE, (j + 1) * CIRCUIT_SIZE)
            .map(v => convertValidator(v, ssz)),
          withdrawalCredentials: bigint_to_array(
            63,
            5,
            computeNumberFromLittleEndianBits(
              hexToBits(
                '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
              ),
            ),
          ),
        }),
      });
    }

    await redis.saveValidatorBalancesInput(batch);

    console.log('saved batch', i);
  }

  for (let i = 0; i < TAKE / CIRCUIT_SIZE; i++) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i * CIRCUIT_SIZE), false);

    await queues[0].addItem(db, new Item(buffer));
    console.log(`added ${i * CIRCUIT_SIZE}`);
  }

  for (let j = 1; j < 39; j++) {
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

      console.log(`added ${j}:${first}:${second}`);

      view.setBigUint64(0, BigInt(j - 1), false);
      view.setBigUint64(8, first, false);
      view.setBigUint64(16, second, false);

      await redis.saveBalanceProof(BigInt(j - 1), first);
      await queues[j].addItem(db, new Item(buffer));

      prev_index = first;
    }
  }

  console.log('ready');

  process.exit(0);
})();

function convertValidator(validator, ssz): any {
  return {
    pubkey: bigint_to_array(
      63,
      7,
      computeNumberFromLittleEndianBits(
        hexToBits(bytesToHex(validator.pubkey), 384),
      ),
    ),
    withdrawalCredentials: bigint_to_array(
      63,
      5,
      computeNumberFromLittleEndianBits(
        hexToBits(bytesToHex(validator.withdrawalCredentials)),
      ),
    ),
    effectiveBalance: bigint_to_array(
      63,
      2,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.effectiveBalance.hashTreeRoot(
              validator.effectiveBalance,
            ),
          ),
        ),
      ),
    ),
    slashed: bigint_to_array(
      63,
      1,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.slashed.hashTreeRoot(validator.slashed),
          ),
        ),
      ),
    ),
    activationEligibilityEpoch: bigint_to_array(
      63,
      2,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.activationEligibilityEpoch.hashTreeRoot(
              validator.activationEligibilityEpoch,
            ),
          ),
        ),
      ),
    ),
    activationEpoch: bigint_to_array(
      63,
      2,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.activationEpoch.hashTreeRoot(
              validator.activationEpoch,
            ),
          ),
        ),
      ),
    ),
    exitEpoch: bigint_to_array(
      63,
      2,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.exitEpoch.hashTreeRoot(
              validator.exitEpoch,
            ),
          ),
        ),
      ),
    ),
    withdrawableEpoch: bigint_to_array(
      63,
      2,
      computeNumberFromLittleEndianBits(
        hexToBits(
          bytesToHex(
            ssz.phase0.Validator.fields.withdrawableEpoch.hashTreeRoot(
              validator.withdrawableEpoch,
            ),
          ),
        ),
      ),
    ),
  };
}

function computeNumberFromLittleEndianBits(bits) {
  let sum = 0n;
  for (let i = 0; i < bits.length; i++) {
    sum += BigInt(bits[i]) * 2n ** BigInt(i);
  }
  return sum;
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
