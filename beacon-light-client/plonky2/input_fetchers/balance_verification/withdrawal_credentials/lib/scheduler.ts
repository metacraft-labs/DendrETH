import chalk from 'chalk';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { KeyPrefix, WorkQueue, Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../../../../kv_db_constants.json';
import { computeEpochAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import {
  convertValidatorToValidatorInput,
  getDummyValidatorInput,
} from '../../common';
import commonConfig from '../../../../common_config.json';

const commonConfigChecked = commonConfig satisfies CommonConfig;

const VALIDATORS_COUNT = 8;

export type GetBalancesInputConfigRequiredFields = {
  withdrawalCredentials: string;
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

  const withdrawalCredentials = config.withdrawalCredentials.padEnd(64, '0');
  const protocol = config.protocol;

  const queues: any[] = [];

  for (let i = 0; i < 38; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(`${protocol}:${CONSTANTS.balanceVerificationQueue}:${i}`),
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
  // TODO: this is wrong, fix it
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

  const balancesLeaves: string[] = [];

  for (let i = 0; i < take / 4; i++) {
    balancesLeaves.push(
      bytesToHex(balancesTree.getNode(balanceZeroIndex + BigInt(i)).root),
    );
  }

  if (balancesLeaves.length % (VALIDATORS_COUNT / 4) !== 0) {
    balancesLeaves.push(''.padStart(64, '0'));
  }

  await redis.saveValidatorBalancesInput(protocol, [
    {
      index: Number(CONSTANTS.validatorRegistryLimit),
      input: {
        balancesLeaves: Array(VALIDATORS_COUNT / 4)
          .fill('')
          .map(() => ''.padStart(64, '0')),
        validators: Array(VALIDATORS_COUNT).fill(
          getDummyValidatorInput(),
        ),
        withdrawalCredentials: [withdrawalCredentials],
        currentEpoch: computeEpochAt(beaconState.slot).toString(),
        nonZeroValidatorLeavesMask: Array(VALIDATORS_COUNT).fill(false),
      },
    },
  ]);

  console.log(chalk.bold.blue('Adding zero tasks...'));

  const buffer = new ArrayBuffer(8);
  const dataView = new DataView(buffer);

  dataView.setBigUint64(0, BigInt(CONSTANTS.validatorRegistryLimit), false);

  await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));

  for (let i = 0; i < 37; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, BigInt(CONSTANTS.validatorRegistryLimit), false);

    await queues[i + 1].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  console.log(chalk.bold.blue('Saving validator balance input...'));
  const batchSize = 100;
  for (let i = 0; i <= take / VALIDATORS_COUNT / batchSize; i++) {
    let batch: any[] = [];
    for (
      let j = i * batchSize;
      j < i * batchSize + batchSize && j < take / VALIDATORS_COUNT;
      j++
    ) {
      let size =
        (j + 1) * VALIDATORS_COUNT <= validators.length
          ? VALIDATORS_COUNT
          : validators.length - j * VALIDATORS_COUNT;

      let array = new Array(size).fill(true);

      batch.push({
        index: j,
        input: {
          balancesLeaves: balancesLeaves.slice(
            j * (VALIDATORS_COUNT / 4),
            (j + 1) * (VALIDATORS_COUNT / 4),
          ),
          validators: [
            ...validators
              .slice(j * VALIDATORS_COUNT, (j + 1) * VALIDATORS_COUNT)
              .map(v => convertValidatorToValidatorInput(v)),
            ...Array(
              (j + 1) * VALIDATORS_COUNT -
              Math.min((j + 1) * VALIDATORS_COUNT, validators.length),
            ).fill(getDummyValidatorInput()),
          ],
          withdrawalCredentials: [withdrawalCredentials],
          currentEpoch: computeEpochAt(beaconState.slot).toString(),
          nonZeroValidatorLeavesMask: array.concat(
            new Array(VALIDATORS_COUNT - size).fill(false),
          ),
        },
      });
    }

    await redis.saveValidatorBalancesInput(protocol, batch);
  }

  await redis.saveBalanceProof(
    protocol,
    0n,
    BigInt(CONSTANTS.validatorRegistryLimit),
  );

  for (let i = 0; i < take / VALIDATORS_COUNT; i++) {
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
      BigInt(CONSTANTS.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(take / VALIDATORS_COUNT / 2 ** level)).keys(),
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
  const stateRootBranch = beaconBlockHeaderTree
    .getSingleProof(
      ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
    )
    .map(bytesToHex);

  const validatorsLengthSSZ = bytesToHex(ssz.UintNum64.hashTreeRoot(take));

  const validatorsBranch = [validatorsLengthSSZ].concat(
    beaconStateTree
      .getSingleProof(ssz.deneb.BeaconState.getPathInfo(['validators']).gindex)
      .map(bytesToHex),
  );

  const balancesBranch = [validatorsLengthSSZ].concat(
    beaconStateTree
      .getSingleProof(ssz.deneb.BeaconState.getPathInfo(['balances']).gindex)
      .map(bytesToHex),
  );

  console.log(chalk.bold.blue('Adding final proof input...'));
  await redis.saveFinalProofInput(protocol, {
    stateRoot: bytesToHex(currentSSZFork.BeaconState.hashTreeRoot(beaconState)),
    stateRootBranch,
    blockRoot: bytesToHex(
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(beaconBlockHeader),
    ),
    slot: beaconState.slot.toString(),
    slotBranch: beaconStateTree.getSingleProof(34n).map(x => bytesToHex(x)),
    balancesBranch,
    validatorsBranch,
  });

  console.log(chalk.bold.greenBright('Done'));

  await redis.quit();
}
