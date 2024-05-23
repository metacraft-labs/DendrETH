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
import {
  convertValidatorToValidatorPoseidonInput,
  getZeroValidatorPoseidonInput,
} from './utils';
import commonConfig from '../common_config.json';

const commonConfigChecked = commonConfig satisfies CommonConfig;

const CIRCUIT_SIZE = 8;

export type GetBalancesInputConfigRequiredFields = {
  withdrawCredentials: string;
  protocol: string;
};

export type GetBalancesInputConfig = GetBalancesInputConfigRequiredFields & {
  beaconNodeUrls: string[];
  slot?: number;
  take: number;
  offset?: number;
  redisHost: string;
  redisPort: number;
};

export type GetBalancesInputParameterType =
  GetBalancesInputConfigRequiredFields & Partial<GetBalancesInputConfig>;

function getDefaultBalancesConfig(): Omit<
  GetBalancesInputConfig,
  keyof GetBalancesInputConfigRequiredFields
> {
  return {
    beaconNodeUrls: commonConfigChecked['beacon-node'],
    slot: undefined,
    take: Infinity,
    offset: undefined,
    redisHost: commonConfigChecked['redis-host'],
    redisPort: Number(commonConfigChecked['redis-port']),
  };
}

export async function getBalancesInput(options: GetBalancesInputParameterType) {
  const config = { ...getDefaultBalancesConfig(), ...options };

  const { ssz } = await import('@lodestar/types');
  const redis = new RedisLocal(config.redisHost, config.redisPort);

  const withdrawCredentials = config.withdrawCredentials;
  const protocol = config.protocol;

  const queues: any[] = [];

  for (let i = 0; i < 38; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${protocol}:${validator_commitment_constants.balanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }

  const beaconApi = await getBeaconApi(config.beaconNodeUrls);

  const slot =
    config.slot !== undefined
      ? BigInt(config.slot)
      : await beaconApi.getHeadSlot();
  const { beaconState } =
    (await beaconApi.getBeaconState(slot)) ||
    panic('Could not fetch beacon state');

  const currentSSZFork = await beaconApi.getCurrentSSZ(slot);

  const offset = Number(config.offset) || 0;
  let take = config.take !== Infinity ? config.take + offset : Infinity;
  const validators = beaconState.validators.slice(offset, take);
  beaconState.balances = beaconState.balances.slice(offset, take);
  beaconState.validators = validators;

  take = validators.length;

  const balancesView = currentSSZFork.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );

  const balancesTree = new Tree(balancesView.node);

  const balanceZeroIndex =
    currentSSZFork.BeaconState.fields.balances.getPathInfo([0]).gindex;

  const balances: number[][] = [];

  for (let i = 0; i < take / 4; i++) {
    balances.push(
      hexToBits(
        bytesToHex(balancesTree.getNode(balanceZeroIndex + BigInt(i)).root),
      ),
    );
  }

  if (balances.length % (CIRCUIT_SIZE / 4) !== 0) {
    balances.push(''.padStart(256, '0').split('').map(Number));
  }

  await redis.saveValidatorBalancesInput(protocol, [
    {
      index: Number(validator_commitment_constants.validatorRegistryLimit),
      input: {
        balances: Array(CIRCUIT_SIZE / 4)
          .fill('')
          .map(() => ''.padStart(256, '0').split('').map(Number)),
        validators: Array(CIRCUIT_SIZE).fill(getZeroValidatorPoseidonInput()),
        withdrawalCredentials: [hexToBits(withdrawCredentials)],
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
  for (let i = 0; i <= take / CIRCUIT_SIZE / batchSize; i++) {
    let batch: any[] = [];
    for (
      let j = i * batchSize;
      j < i * batchSize + batchSize && j < take / CIRCUIT_SIZE;
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
          withdrawalCredentials: [hexToBits(withdrawCredentials)],
          currentEpoch: computeEpochAt(beaconState.slot).toString(),
          validatorIsZero: array.concat(new Array(CIRCUIT_SIZE - size).fill(1)),
        },
      });
    }

    await redis.saveValidatorBalancesInput(protocol, batch);
  }

  await redis.saveBalanceProof(
    protocol,
    0n,
    BigInt(validator_commitment_constants.validatorRegistryLimit),
  );

  for (let i = 0; i < take / CIRCUIT_SIZE; i++) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i), false);

    await redis.saveBalanceProof(protocol, 0n, BigInt(i));

    await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level < 38; level++) {
    await redis.saveBalanceProof(
      protocol,
      BigInt(level),
      BigInt(validator_commitment_constants.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(take / CIRCUIT_SIZE / 2 ** level)).keys(),
    ];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveBalanceProof(protocol, BigInt(level), BigInt(key));

      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(Buffer.from(buffer)));
    }
  }

  const beaconStateView = currentSSZFork.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

  const beaconBlockHeader = await beaconApi.getBlockHeader(slot);
  beaconBlockHeader.stateRoot =
    currentSSZFork.BeaconState.hashTreeRoot(beaconState);

  const beaconBlockHeaderView =
    ssz.phase0.BeaconBlockHeader.toViewDU(beaconBlockHeader);
  const beaconBlockHeaderTree = new Tree(beaconBlockHeaderView.node);
  const stateRootProof = beaconBlockHeaderTree
    .getSingleProof(
      ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
    )
    .map(bytesToHex);

  console.log(chalk.bold.blue('Adding final proof input...'));
  await redis.saveFinalProofInput(protocol, {
    stateRoot: hexToBits(
      bytesToHex(currentSSZFork.BeaconState.hashTreeRoot(beaconState)),
    ),
    stateRootBranch: stateRootProof.map(x => hexToBits(x)),
    blockRoot: hexToBits(
      bytesToHex(ssz.phase0.BeaconBlockHeader.hashTreeRoot(beaconBlockHeader)),
    ),
    slot: beaconState.slot.toString(),
    slotBranch: beaconStateTree
      .getSingleProof(34n)
      .map(x => hexToBits(bytesToHex(x))),
    withdrawalCredentials: [hexToBits(withdrawCredentials)],
    balanceBranch: beaconStateTree
      .getSingleProof(44n)
      .map(x => hexToBits(bytesToHex(x))),

    validatorsBranch: beaconStateTree
      .getSingleProof(43n)
      .map(x => hexToBits(bytesToHex(x))),
    validatorsSizeBits: hexToBits(bytesToHex(ssz.UintNum64.hashTreeRoot(take))),
  });

  console.log(chalk.bold.greenBright('Done'));

  await redis.quit();
}
