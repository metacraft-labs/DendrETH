import yargs from 'yargs';
import { Redis as RedisLocal } from '../../../relay/implementations/redis';
import config from '../common_config.json';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import CONSTANTS from '../constants/validator_commitment_constants.json';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { computeEpochAt } from '../../../libs/typescript/ts-utils/ssz-utils';
import { readFileSync } from 'fs';
import {
  convertValidatorToProof,
  getCommitmentMapperProof,
  getNthParent,
  getZeroValidatorInput,
  gindexFromIndex,
} from '../validators_commitment_mapper_tree/utils';
import { splitIntoBatches } from '../../../libs/typescript/ts-utils/common-utils';
import {
  BalancesAccumulatorInput,
  Validator,
  ValidatorPoseidonInput,
} from '../../../relay/types/types';
import Redis from 'ioredis';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import {
  convertValidatorToValidatorPoseidonInput,
  getZeroValidatorPoseidonInput,
} from './utils';
import chalk from 'chalk';

const CIRCUIT_SIZE = 2;
let TAKE: number

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
      default: Infinity,
      description: 'Sets the number of validators to take',
    })
    .options('offset', {
      alias: 'offset',
      describe: 'Index offset in the validator set',
      type: 'number',
      default: 0,
    })
    .options('protocol', {
      alias: 'protocol',
      describe: 'The protocol',
      type: 'string',
      default: 'demo',
      description: 'Sets the protocol',
    }).argv;

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);
  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

  TAKE = options['take'];

  const queues: any[] = [];

  for (let i = 0; i < 38; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${CONSTANTS.balanceVerificationAccumulatorProofQueue}:${i}`,
        ),
      ),
    );
  }

  const beaconApi = new BeaconApi([options['beacon-node']]);
  const slot = 8669632;
  // const slot =
  //   options['slot'] !== undefined
  //     ? options['slot']
  //     : Number(await beaconApi.getHeadSlot());

  const { beaconState } = await beaconApi.getBeaconState(slot);

  const offset = Number(options['offset']) || 0;

  beaconState.balances = beaconState.balances.slice(offset, offset + TAKE);
  beaconState.validators = beaconState.validators.slice(offset, offset + TAKE);

  let validatorsAccumulator: any[] = JSON.parse(
    readFileSync('validators.json', 'utf-8'),
  );

  validatorsAccumulator.map((x, i) => {
    x.validator_index = i;
    return x;
  });

  // Should be the balances of the validators
  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const balancesTree = new Tree(balancesView.node);

  let balancesProofs = validatorsAccumulator.map(v => {
    return balancesTree
      .getSingleProof(gindexFromIndex(BigInt(v.validator_index) / 4n, 39n))
      .map(bytesToHex)
      .slice(0, 22);
  });

  // get validator commitment root from redis
  let validatorCommitmentRoot = await redis.getValidatorCommitmentRoot(
    computeEpochAt(beaconState.slot),
  );
  if (validatorCommitmentRoot === null) {
    throw new Error(`Validator root for epoch ${computeEpochAt(beaconState.slot)} is missing`);
  }

  // load proofs for the validators from redis
  let validatorCommitmentProofs = await Promise.all(
    validatorsAccumulator.map(async v => {
      return (
        await getCommitmentMapperProof(
          BigInt(computeEpochAt(beaconState.slot)),
          gindexFromIndex(BigInt(v.validator_index), 40n),
          'poseidon',
          redis,
        )
      ).slice(0, 24);
    }),
  );

  let balancesInputs: BalancesAccumulatorInput[] = [];
  for (
    let chunkIdx = 0;
    chunkIdx <
    Math.floor(
      (validatorsAccumulator.length + CIRCUIT_SIZE - 1) / CIRCUIT_SIZE,
    );
    chunkIdx++
  ) {
    let balancesInput: BalancesAccumulatorInput = {
      balancesRoot: bytesToHex(balancesTree.getRoot(2n ** 17n)), // NOTE: this is probably wrong
      balances: [],
      balancesProofs: [],
      validatorDepositIndexes: [],
      validatorsGindices: [],
      validatorCommitmentProofs: [],
      validatorIsNotZero: [],
      validators: [],
      validatorCommitmentRoot: validatorCommitmentRoot!,
      currentEpoch: computeEpochAt(beaconState.slot),
      currentEth1DepositIndex: beaconState.eth1DepositIndex,
    };
    for (let j = 0; j < CIRCUIT_SIZE; j++) {
      const idx = chunkIdx * CIRCUIT_SIZE + j;
      if (idx < validatorsAccumulator.length) {
        balancesInput.balances.push(
          bytesToHex(
            balancesTree.getNode(gindexFromIndex(BigInt(validatorsAccumulator[idx].validator_index) / 4n, 39n)).root, // tva mai e greshno
          ),
        );
        balancesInput.balancesProofs.push(balancesProofs[idx]);
        balancesInput.validatorDepositIndexes.push(
          validatorsAccumulator[idx].validator_index,
        );
        balancesInput.validatorsGindices.push(
          Number(gindexFromIndex(BigInt(validatorsAccumulator[idx].validator_index), 24n)),
        );
        balancesInput.validators.push(
          convertValidatorToValidatorPoseidonInput(beaconState.validators[idx]),
        );
        balancesInput.validatorCommitmentProofs.push(
          validatorCommitmentProofs[idx],
        );
        balancesInput.validatorIsNotZero.push(1);
      } else {
        balancesInput.balances.push(''.padStart(64, '0'));
        balancesInput.balancesProofs.push(
          new Array(22).map(x => ''.padStart(64, '0')),
        );
        balancesInput.validators.push(getZeroValidatorPoseidonInput());
        balancesInput.validatorDepositIndexes.push(0);
        balancesInput.validatorsGindices.push(0);
        balancesInput.validatorCommitmentProofs.push(
          new Array(22).map(x => new Array(4).fill(0)),
        );
        balancesInput.validatorIsNotZero.push(0);
      }
    }

    balancesInputs.push(balancesInput);
  }

  // first level tasks
  await redis.saveBalancesAccumulatorInput(balancesInputs, options['protocol']);
  await redis.saveBalancesAccumulatorProof(options['protocol'], 0n, BigInt(CONSTANTS.validatorRegistryLimit));
  await scheduleFirstLevelTasks(queues[0], balancesInputs);

  // inner level tasks
  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level < 24; level++) {
    await redis.saveBalancesAccumulatorProof(
      options['protocl'],
      BigInt(level),
      BigInt(CONSTANTS.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(TAKE / CIRCUIT_SIZE / 2 ** level)).keys(),
    ];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveBalancesAccumulatorProof(options['protocol'], BigInt(level), BigInt(key));
      // schedule tasks

      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(buffer));
    }
  }

  db.quit();
  await redis.disconnect();

  async function scheduleFirstLevelTasks(queue: any, balancesInputs: BalancesAccumulatorInput[]) {
    for (let i = 0; i < balancesInputs.length; i++) {
      const buffer = new ArrayBuffer(8);
      const dataView = new DataView(buffer);
      dataView.setBigUint64(0, BigInt(i), false);
      queue.addItem(db, new Item(buffer));
    }
  }
})();
