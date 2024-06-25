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
import { getCommitmentMapperProof } from '../../../utils/common_utils';
import ValidatorsAccumulator from '../../../../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';
import { getEvents } from './event_fetcher';
import chalk from 'chalk';
import { queryContractDeploymentBlockNumber } from './utils';

enum Events {
  Deposited = 'Deposited',
}

const CHUNK_SIZE = 10_000;
const MAX_UINT = '18446744073709551615';

const commonConfigChecked = commonConfig satisfies CommonConfig;

export type StoreBalanceVerificationConfigRequiredFields = {
  // withdrawalCredentials: string;
  address: string;
  rpcUrl: string;
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
    redisAuth?: string;
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

  const redis = new RedisLocal(config.redisHost, config.redisPort, options.redisAuth);
  const beaconApi = await getBeaconApi(config.beaconNodeUrls);
  const provider = new ethers.providers.JsonRpcProvider(config.rpcUrl);

  const queues: any[] = [];
  for (let i = 0; i <= 32; i++) {
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

  const firstBlock = await queryContractDeploymentBlockNumber(
    provider,
    config.address,
  );

  if (firstBlock === null) {
    throw new Error('Contract not found');
  }

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
        [Events.Deposited]: ['pubkey'],
      },
      block,
      lastBlockInChunk,
    );

    logs = logs.concat(data);
  }

  console.log(logs.length, 'Deposits found');

  console.log(chalk.bold.blue('Adding zero tasks...'));

  const buffer = new ArrayBuffer(8);
  const dataView = new DataView(buffer);

  dataView.setBigUint64(0, BigInt(CONSTANTS.validatorRegistryLimit), false);

  await queues[0].addItem(redis.client, new Item(Buffer.from(buffer)));

  for (let i = 0; i < 31; i++) {
    const buffer = new ArrayBuffer(24);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, BigInt(CONSTANTS.validatorRegistryLimit), false);

    await queues[i + 1].addItem(redis.client, new Item(Buffer.from(buffer)));
  }

  const { ssz } = await import('@lodestar/types');
  const balancesView = ssz.deneb.BeaconState.fields.balances.toViewDU(
    beaconState.balances,
  );
  const balancesTree = new Tree(balancesView.node);

  await redis.saveDepositBalanceVerificationInput(
    config.protocol,
    BigInt(CONSTANTS.validatorRegistryLimit),
    {
      validator: {
        pubkey: '0'.padStart(96, '0'),
        withdrawalCredentials: '0'.padStart(64, '0'),
        effectiveBalance: '0',
        slashed: false,
        activationEligibilityEpoch: '0',
        activationEpoch: '0',
        exitEpoch: '0',
        withdrawableEpoch: '0',
      },
      depositPubkey: '0'.padStart(96, '0'),
      validatorsCommitmentMapperRoot:
        await redis.extractHashFromCommitmentMapperProof(
          65536n,
          BigInt(beaconState.slot),
          'poseidon',
        ),
      validatorsCommitmentMapperBranch: [...Array(24).keys()].map(x => [
        0, 0, 0, 0,
      ]),
      validatorGindex: '0',
      balancesRoot: bytesToHex(
        balancesTree.getNode(getNthParent(549755813888n, 22n)).root,
      ),
      balanceLeaf: '0'.padStart(64, '0'),
      balanceBranch: [...Array(22).keys()].map(x => '0'.padStart(64, '0')),
      currentEpoch: (BigInt(beaconState.slot) / 32n).toString(),
      isDummy: true,
    },
  );

  console.log(chalk.bold.blue('Saving balance verification inputs...'));
  for (const [i, log] of logs.entries()) {
    const data = log[Events.Deposited];

    await generateLeafLevelData(
      data.pubkey,
      beaconState,
      redis,
      config.protocol,
      i,
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
  }

  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level <= 32; level++) {
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

  console.log(chalk.bold.blue('Adding final proof input...'));

  const currentSSZ = await beaconApi.getCurrentSSZ(slot);

  const beaconBlockHeader = await beaconApi.getBlockHeader(slot);
  beaconBlockHeader.stateRoot =
    currentSSZ.BeaconState.hashTreeRoot(beaconState);

  const beaconBlockHeaderView =
    ssz.phase0.BeaconBlockHeader.toViewDU(beaconBlockHeader);
  const beaconBlockHeaderTree = new Tree(beaconBlockHeaderView.node);
  const stateRootBranch = beaconBlockHeaderTree
    .getSingleProof(
      ssz.phase0.BeaconBlockHeader.getPathInfo(['state_root']).gindex,
    )
    .map(bytesToHex);

  const validatorsLengthSSZ = bytesToHex(
    ssz.UintNum64.hashTreeRoot(beaconState.validators.length),
  );

  const beaconStateView = currentSSZ.BeaconState.toViewDU(beaconState);
  const beaconStateTree = new Tree(beaconStateView.node);

  const validatorsBranch = [validatorsLengthSSZ].concat(
    beaconStateTree
      .getSingleProof(ssz.deneb.BeaconState.getPathInfo(['validators']).gindex)
      .map(bytesToHex),
  );

  const balancesBranch = beaconStateTree
    .getSingleProof(ssz.deneb.BeaconState.getPathInfo(['balances']).gindex)
    .map(bytesToHex);

  let proof = balancesTree.getSingleProof(131072n).map(bytesToHex);

  const executionBlockNumber =
    beaconState.latestExecutionPayloadHeader.blockNumber.toString();

  const executionBlockNumberBranch = beaconStateTree
    .getSingleProof(
      ssz.deneb.BeaconState.getPathInfo([
        'latest_execution_payload_header',
        'block_number',
      ]).gindex,
    )
    .map(bytesToHex);

  await redis.saveBalanceAggregatorFinalProofInput(config.protocol, {
    blockRoot: bytesToHex(
      ssz.phase0.BeaconBlockHeader.hashTreeRoot(beaconBlockHeader),
    ),
    stateRoot: bytesToHex(currentSSZ.BeaconState.hashTreeRoot(beaconState)),
    stateRootBranch,
    validatorsBranch,
    balanceBranch: proof.concat(balancesBranch),
    executionBlockNumber,
    executionBlockNumberBranch,
    slot: beaconState.slot.toString(),
    slotBranch: beaconStateTree.getSingleProof(34n).map(x => bytesToHex(x)),
  });

  console.log(chalk.bold.greenBright('Done'));

  await redis.quit();
}

type BeaconState = Awaited<
  ReturnType<BeaconApi['getBeaconState']>
>['beaconState'];

async function generateLeafLevelData(
  pubkey: string,
  beaconState: BeaconState,
  redis: Redis,
  protocol: string,
  index: number,
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

  const deposit_accumulator_input = {
    validator: dataValidator,
    depositPubkey: formatHex(pubkey),
    validatorsCommitmentMapperRoot:
      await redis.extractHashFromCommitmentMapperProof(
        65536n,
        BigInt(beaconState.slot),
        'poseidon',
      ),
    validatorsCommitmentMapperBranch: (
      await getCommitmentMapperProof(
        BigInt(beaconState.slot),
        gindexFromIndex(BigInt(foundIndex), 40n),
        'poseidon',
        redis,
      )
    ).slice(0, 24),
    validatorGindex: gindexFromIndex(BigInt(foundIndex), 24n).toString(),
    balancesRoot: bytesToHex(
      balancesTree.getNode(
        getNthParent(balanceZeroGindex + BigInt(balanceIndex), 22n),
      ).root,
    ),
    balanceLeaf: bytesToHex(
      balancesTree.getNode(balanceZeroGindex + BigInt(balanceIndex)).root,
    ),
    balanceBranch: balancesTree
      .getSingleProof(balanceZeroGindex + BigInt(balanceIndex))
      .slice(0, 22)
      .map(bytesToHex),
    currentEpoch: (BigInt(beaconState.slot) / 32n).toString(),
    isDummy: false,
  };

  await redis.saveDepositBalanceVerificationInput(
    protocol,
    BigInt(index),
    deposit_accumulator_input,
  );
}
