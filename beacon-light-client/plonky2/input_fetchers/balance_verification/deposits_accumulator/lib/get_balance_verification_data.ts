import { ethers } from 'ethers';
import JSONbig from 'json-bigint';
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
import { verifyMerkleProof } from '@dendreth/utils/ts-utils/ssz-utils';
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

  const beaconBlock = await beaconApi.getBeaconBlock(BigInt(slot));

  const firstBlock = config.syncBlock;
  const lastBlock = beaconBlock?.body.executionPayload.blockNumber!;

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

  for (const [i, log] of logs.entries()) {
    const data = log[Events.Deposited];

    console.log(chalk.bold.blue('Saving balance verification input...'));
    generate_leaf_level_data(
      data.pubkey,
      BigInt(data.depositIndex),
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

  console.log(chalk.bold.blue('Adding inner proofs...'));
  for (let level = 1; level < 38; level++) {
    await redis.saveBalanceProof(
      config.protocol,
      BigInt(level),
      BigInt(CONSTANTS.validatorRegistryLimit),
    );

    const range = [
      ...new Array(Math.ceil(take / logs.length / 2 ** level)).keys(),
    ];
    for (const key of range) {
      const buffer = new ArrayBuffer(8);
      const view = new DataView(buffer);

      await redis.saveBalanceProof(config.protocol, BigInt(level), BigInt(key));

      view.setBigUint64(0, BigInt(key), false);
      await queues[level].addItem(redis.client, new Item(Buffer.from(buffer)));
    }
  }
}

type BeaconState = Awaited<
  ReturnType<BeaconApi['getBeaconState']>
>['beaconState'];

async function generate_leaf_level_data(
  pubkey: string,
  deposit_index: bigint,
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

  const dataValidator = JSONbig.parse(JSONbig.stringify(validator));
  dataValidator.pubkey = bytesToHex(dataValidator.pubkey) as any;
  dataValidator.withdrawalCredentials = bytesToHex(
    dataValidator.withdrawalCredentials,
  ) as any;

  const deposit_accumulator_input = {
    validator: dataValidator,
    validatorDeposit: {
      pubkey,
      depositIndex: deposit_index.toString(),
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
        1n,
        'poseidon',
      ))!.map(toString),
    validatorDepositProof: await getDepositCommitmentMapperProof(
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

  let result = verifyMerkleProof(
    [
      'cbf1a3690b000000798608680b000000cd73407d0b00000001ad6a6b0b000000',
      '34f735cad9ae2d061fbab0682064d1b37e8c227e0f13e07457ce12d69e97da43',
      'efb80785674ab41400abe50d7b3b837128ac54451ae0bf433cb9e4d9cbfc6c4c',
      'c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c',
      '536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c',
      '9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30',
      'd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1',
      '87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c',
      '26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193',
      '506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1',
      'ffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b',
      '6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220',
      'b7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f',
      'df6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e',
      'b58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784',
      'd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb',
      '8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb',
      '8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab',
      '95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4',
      'f893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f',
      'cddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa',
      '8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c',
    ],
    bytesToHex(
      balancesTree.getNode(
        getNthParent(balanceZeroGindex + BigInt(balanceIndex), 22n),
      ).root,
    ),
    'b07ad63907000000045d8b6d0b000000be642c690b0000001cba346c0b000000',
    0n,
  );

  console.log(result);

  await redis.saveDepositBalanceVerificationInput(
    protocol,
    1n,
    BigInt(foundIndex),
    deposit_accumulator_input,
  );
}
