import { ethers } from 'ethers';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { Item, KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import { bytesToHex, formatHex } from '@dendreth/utils/ts-utils/bls';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import {
  getNthParent,
  gindexFromIndex,
} from '@dendreth/utils/ts-utils/common-utils';
import { Redis as RedisLocal } from '@dendreth/relay/implementations/redis';
import { panic } from '@dendreth/utils/ts-utils/common-utils';
import { Redis } from '@dendreth/relay/implementations/redis';
import CONSTANTS from '../../../../kv_db_constants.json';
import commonConfig from '../../../../common_config.json';
import {
  getCommitmentMapperProof,
  getDepositCommitmentMapperProof,
} from '../../../utils/common_utils';
import ValidatorsAccumulator from '../../../../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';
import { getEvents } from './event_fetcher';
import chalk from 'chalk';

enum Events {
  Deposited = 'Deposited',
}

const CHUNK_SIZE = 10_000;
const MAX_UINT = '18446744073709551615';

const commonConfigChecked = commonConfig satisfies CommonConfig;

export type StoreBalanceVerificationConfigRequiredFields = {
  withdrawalCredentials: string;
  address: string;
  rpcUrl: string;
  syncBlock: number;
  protocol: string;
};

export type StoreBalanceVerificationConfig =
  StoreBalanceVerificationConfigRequiredFields & {
    beaconNodeUrls: string[];
    slot?: number;
    take: number;
    offset?: number;
    redisHost: string;
    redisPort: number;
  };

export type StoreBalanceVerificationParameterType =
  StoreBalanceVerificationConfigRequiredFields &
    Partial<StoreBalanceVerificationConfig>;

function getDefaultBalanceVerificationConfig(): Omit<
  StoreBalanceVerificationConfig,
  keyof StoreBalanceVerificationConfigRequiredFields
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

export async function storeBalanceVerificationData(
  options: StoreBalanceVerificationParameterType,
) {
  const config = { ...getDefaultBalanceVerificationConfig(), ...options };

  const redis = new RedisLocal(config.redisHost, config.redisPort);
  const beaconApi = await getBeaconApi(config.beaconNodeUrls);
  const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);

  const queues: any[] = [];
  for (let i = 0; i < 38; i++) {
    queues.push(
      new WorkQueue(
        new KeyPrefix(
          `${config.protocol}:${CONSTANTS.depositBalanceVerificationQueue}:${i}`,
        ),
      ),
    );
  }

  const slot =
    config.slot !== undefined
      ? BigInt(config.slot)
      : await beaconApi.getHeadSlot();
  const { beaconState } =
    (await beaconApi.getBeaconState(slot)) ||
    panic('Could not fetch beacon state');

  const offset = Number(config.offset) || 0;
  let take = config.take !== Infinity ? config.take + offset : Infinity;
  beaconState.balances = beaconState.balances.slice(offset, take);
  beaconState.validators = beaconState.validators.slice(offset, take);

  const firstBlock = config.syncBlock;
  const lastBlock = beaconState.latestExecutionPayloadHeader.blockNumber!;

  const contract = new ethers.Contract(
    config.address,
    ValidatorsAccumulator.abi,
    provider,
  );

  let logs: any[] = [];
  for (let block = firstBlock; block <= lastBlock; block += CHUNK_SIZE) {
    const lastBlockInChunk = Math.min(block + CHUNK_SIZE - 1, lastBlock);
    const data = await getEvents(
      provider,
      contract,
      {
        [Events.Deposited]: [
          'pubkey',
          'depositIndex',
          'signature',
          'depositMessageRoot',
        ],
      },
      block,
      lastBlockInChunk,
    );

    logs = logs.concat(data);
  }

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

  console.log(chalk.bold.blue('Saving balance verification inputs...'));
  for (const [i, log] of logs.entries()) {
    const data = log[Events.Deposited];

    await generate_leaf_level_data(
      data.pubkey,
      data.depositIndex,
      data.signature,
      data.depositMessageRoot,
      beaconState,
      redis,
      config.protocol,
    );

    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i), false);

    await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  take = logs.length;

  await redis.saveDepositBalanceVerificationProof(
    config.protocol,
    0n,
    BigInt(CONSTANTS.validatorRegistryLimit),
  );

  for (let i = 0; i < take; i++) {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(i), false);

    await redis.saveDepositBalanceVerificationProof(
      config.protocol,
      0n,
      BigInt(i),
    );

    await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level < 38; level++) {
    await redis.saveDepositBalanceVerificationProof(
      config.protocol,
      BigInt(level),
      BigInt(CONSTANTS.validatorRegistryLimit),
    );

    const range = [...new Array(Math.ceil(take / 2 ** level)).keys()];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveDepositBalanceVerificationProof(
        config.protocol,
        BigInt(level),
        BigInt(key),
      );

      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(Buffer.from(buffer)));
    }
  }

  console.log(chalk.bold.greenBright('Done'));
  await redis.quit();
}

type BeaconState = Awaited<
  ReturnType<BeaconApi['getBeaconState']>
>['beaconState'];

async function generate_leaf_level_data(
  pubkey: string,
  deposit_index: string,
  signature: string,
  deposit_message_hash_tree_root: string,
  beaconState: BeaconState,
  redis: Redis,
  protocol: string,
) {
  const { ssz } = await import('@lodestar/types');

  let foundIndex = -1;
  const validator = beaconState.validators.find((validator, i) => {
    if (formatHex(bytesToHex(validator.pubkey)) === formatHex(pubkey)) {
      foundIndex = i;
      return true;
    }
    return false;
  });

  if (foundIndex === -1) {
    throw new Error('Validator not found');
  }

  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );
  const balancesTree = new Tree(balancesView.node);
  const balanceZeroGindex = ssz.deneb.BeaconState.fields.balances.getPathInfo([
    0,
  ]).gindex;

  const balanceIndex = Math.floor(foundIndex / 4);
  const dataValidator = { ...validator };
  dataValidator.pubkey = bytesToHex(validator!.pubkey) as any;
  dataValidator.withdrawalCredentials = bytesToHex(
    validator!.withdrawalCredentials,
  ) as any;

  for (const key of Object.keys(dataValidator)) {
    if (dataValidator[key] === Infinity) {
      dataValidator[key] = MAX_UINT;
    }
    if (typeof dataValidator[key] === 'number') {
      dataValidator[key] = dataValidator[key].toString();
    }
  }

  const index = parseInt(
    '0x' + deposit_index.replace('0x', '').match(/../g)?.reverse().join(''),
  );

  const deposit_accumulator_input = {
    validator: dataValidator,
    validatorDeposit: {
      pubkey,
      depositIndex: index.toString(),
      signature,
      depositMessageRoot: deposit_message_hash_tree_root,
    },
    commitmentMapperRoot: (await redis.extractHashFromCommitmentMapperProof(
      1n,
      BigInt(beaconState.slot),
      'poseidon',
    ))!.map(toString),
    commitmentMapperProof: await getCommitmentMapperProof(
      BigInt(beaconState.slot),
      gindexFromIndex(BigInt(foundIndex), 40n),
      'poseidon',
      redis,
    ),
    validatorIndex: foundIndex,
    validatorDepositRoot:
      (await redis.extractHashFromDepositCommitmentMapperProof(
        protocol,
        1n,
        'poseidon',
      ))!.map(toString),
    validatorDepositProof: await getDepositCommitmentMapperProof(
      protocol,
      gindexFromIndex(BigInt(foundIndex), 40n),
      'poseidon',
      redis,
    ),
    balanceTreeRoot: bytesToHex(
      balancesTree.getNode(
        getNthParent(balanceZeroGindex + BigInt(balanceIndex), 22n),
      ).root,
    ),
    balanceLeaf: bytesToHex(
      balancesTree.getNode(balanceZeroGindex + BigInt(balanceIndex)).root,
    ),
    balanceProof: balancesTree
      .getSingleProof(balanceZeroGindex + BigInt(balanceIndex))
      .slice(0, 22)
      .map(bytesToHex),
    blsSignatureProofKey: `bls12_381_${pubkey}_${deposit_index}`,
    currentEpoch: (BigInt(beaconState.slot) / 32n).toString(),
    isDummy: false,
    eth1DepositIndex: beaconState.eth1DepositIndex,
  };

  await redis.saveDepositBalanceVerificationInput(
    protocol,
    BigInt(foundIndex),
    deposit_accumulator_input,
  );
}
