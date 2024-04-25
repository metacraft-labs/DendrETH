import { Redis as RedisLocal } from '../../../relay/implementations/redis';
const {
  KeyPrefix,
  WorkQueue,
  Item,
} = require('@mevitae/redis-work-queue/dist/WorkQueue');
import CONSTANTS from '../constants/validator_commitment_constants.json';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { computeEpochAt } from '../../../libs/typescript/ts-utils/ssz-utils';
import {
  getCommitmentMapperProof,
  gindexFromIndex,
} from '../validators_commitment_mapper_tree/utils';
import {
  BalancesAccumulatorInput,
  DepositData,
} from '../../../relay/types/types';
import Redis from 'ioredis';
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import {
  convertValidatorToValidatorPoseidonInput,
  getZeroValidatorPoseidonInput,
} from './utils';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import { CommandLineOptionsBuilder } from '../cmdline';

import deposits from './deposits.json';
deposits satisfies DepositData[];

const CIRCUIT_SIZE = 2;

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .withBeaconNodeOpts()
    .withProtocolOpts()
    .withRangeOpts()
    .option('slot', {
      alias: 'slot',
      describe: 'The state slot',
      type: 'number',
      default: undefined,
      description: 'Fetches the balances for this slot',
    }).build();

  const redis = new RedisLocal(options['redis-host'], options['redis-port']);
  const db = new Redis(
    `redis://${options['redis-host']}:${options['redis-port']}`,
  );

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

  const beaconApi = new BeaconApi(options['beacon-node'], ssz);
  const slot = 8915136n;
  // const slot =
  //   options['slot'] !== undefined
  //     ? options['slot']
  //     : Number(await beaconApi.getHeadSlot());

  const { beaconState } = await beaconApi.getBeaconState(slot) || panic("Could not get beacon state");
  // const beaconBlock = await beaconApi.getBeaconBlock(slot) || panic("Could not get beacon block");
  // const deposits = beaconBlock.body.deposits.map(deposit => deposit.data);
  // const depositsInput = deposits.map(deposit => ({
  //   pubkey: bytesToHex(deposit.pubkey),
  //   withdrawalCredentials: bytesToHex(deposit.withdrawalCredentials),
  //   amount: deposit.amount,
  //   signature: bytesToHex(deposit.signature),
  // }));

  beaconState.balances = beaconState.balances.slice(0, deposits.length);
  beaconState.validators = beaconState.validators.slice(0, deposits.length);

  const validatorIndices = [...deposits].map((_, index) => index);

  // Should be the balances of the validators
  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const balancesTree = new Tree(balancesView.node);

  let balancesProofs = validatorIndices.map((index) => {
    return balancesTree
      .getSingleProof(gindexFromIndex(BigInt(index) / 4n, 39n))
      .map(bytesToHex)
      .slice(0, 22);
  });

  // get validator commitment root from redis
  let validatorCommitmentRoot = await redis.getValidatorsCommitmentRoot(slot);
  if (validatorCommitmentRoot === null) {
    throw new Error(`Validator root for slot ${slot} is missing`);
  }

  // load proofs for the validators from redis
  let validatorCommitmentProofs = await Promise.all(
    validatorIndices.map(async index => {
      return (
        await getCommitmentMapperProof(
          BigInt(computeEpochAt(beaconState.slot)),
          gindexFromIndex(BigInt(index), 40n),
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
      (deposits.length + CIRCUIT_SIZE - 1) / CIRCUIT_SIZE,
    );
    chunkIdx++
  ) {
    let balancesInput: BalancesAccumulatorInput = {
      balancesRoot: bytesToHex(balancesTree.getRoot(2n ** 17n)), // NOTE: this is probably wrong
      balancesLeaves: [],
      balancesProofs: [],
      validatorDepositIndexes: [],
      validatorIndices: [],
      validatorCommitmentProofs: [],
      validatorIsNotZero: [],
      validators: [],
      validatorCommitmentRoot: validatorCommitmentRoot!,
      currentEpoch: computeEpochAt(beaconState.slot),
      currentEth1DepositIndex: beaconState.eth1DepositIndex,
      depositsData: [],
      validatorsPoseidonRoot: [2, 2, 2, 2], // TODO: supply the real validators root
    };
    for (let j = 0; j < CIRCUIT_SIZE; j++) {
      const idx = chunkIdx * CIRCUIT_SIZE + j;

      if (idx < deposits.length) {
        balancesInput.balancesLeaves.push(
          bytesToHex(
            balancesTree.getNode(gindexFromIndex(BigInt(validatorIndices[idx]) / 4n, 39n)).root,
          ),
        );
        balancesInput.balancesProofs.push(balancesProofs[idx]);
        balancesInput.validatorDepositIndexes.push( // TODO: delete this, we get this from the deposit
          validatorIndices[idx],
        );
        balancesInput.validatorIndices.push(
          // Number(gindexFromIndex(BigInt(validatorsAccumulator[idx].validator_index), 24n)),
          idx
        );
        balancesInput.validators.push(
          convertValidatorToValidatorPoseidonInput(beaconState.validators[idx]),
        );
        balancesInput.validatorCommitmentProofs.push(
          validatorCommitmentProofs[idx],
        );
        balancesInput.validatorIsNotZero.push(1);
        balancesInput.depositsData.push(deposits[idx]);
      } else {
        balancesInput.balancesLeaves.push(''.padStart(64, '0'));
        balancesInput.balancesProofs.push(
          new Array(22).map(() => ''.padStart(64, '0')),
        );
        balancesInput.validators.push(getZeroValidatorPoseidonInput());
        balancesInput.validatorDepositIndexes.push(0);
        balancesInput.validatorIndices.push(0);
        balancesInput.validatorCommitmentProofs.push(
          new Array(22).map(() => new Array(4).fill(0)),
        );
        balancesInput.validatorIsNotZero.push(0);
        balancesInput.depositsData.push({
          pubkey: ''.padStart(48, '0'),
          withdrawalCredentials: ''.padStart(32, '0'),
          amount: 0,
          signature: ''.padStart(96, '0'),
        });
      }
    }

    balancesInputs.push(balancesInput);
  }

  // first level tasks
  await redis.saveBalancesAccumulatorInput(balancesInputs, options['protocol']);
  await redis.saveBalancesAccumulatorProof(options['protocol'], 0n, BigInt(CONSTANTS.validatorRegistryLimit));
  await scheduleFirstLevelTasks(queues[0], balancesInputs);

  // inner level tasks
  for (let level = 1; level < 24; level++) {
    await redis.saveBalancesAccumulatorProof(
      options['protocl'],
      BigInt(level),
      BigInt(CONSTANTS.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(deposits.length / CIRCUIT_SIZE / 2 ** level)).keys(),
    ];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveBalancesAccumulatorProof(options['protocol'], BigInt(level), BigInt(key));

      // schedule task
      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(buffer));
    }
  }

  db.quit();
  await redis.quit();

  async function scheduleFirstLevelTasks(queue: any, balancesInputs: BalancesAccumulatorInput[]) {
    for (let i = 0; i < balancesInputs.length; i++) {
      const buffer = new ArrayBuffer(8);
      const dataView = new DataView(buffer);
      dataView.setBigUint64(0, BigInt(i), false);
      queue.addItem(db, new Item(buffer));
    }
  }
})();
