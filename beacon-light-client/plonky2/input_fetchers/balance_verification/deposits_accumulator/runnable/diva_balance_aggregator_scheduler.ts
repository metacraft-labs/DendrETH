import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import runTask, { retry } from '../../../utils/ecs';
import accountManagerAbi from '../../../abi/account_manager_abi.json';
import { BigNumber, ethers } from 'ethers';
import { storeBalanceVerificationData } from '../lib/get_balance_verification_data';
import { Redis } from '@dendreth/relay/implementations/redis';
import CONSTANTS from '../../../../kv_db_constants.json';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import JSONbig from 'json-bigint';
import 'dotenv/config';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { lightCleanQueue } from './balance_aggregator_light_cleaner';

const MAX_INSTANCES: number = 10; // TODO

function level(n: number, w: number, d: number): number {
  return Math.ceil(n / w) * d;
}

function predict(n: number, w: number): number {
  let ans = level(n, w, 30);
  for (let i = 1; i <= 31; i++) {
    const v = Math.ceil(n / 2 ** i);
    const x = level(v, w, 1.5);
    ans += x;
  }
  return ans;
}

// Estimate how many workers we'd need to compute the task under 50 minutes.
function estimate(n: number, t: number = 3000): number {
  let low = 1;
  let high = 2 ** 10;
  while (low <= high) {
    const w = Math.floor((low + high) / 2);
    const x = predict(n, w);
    if (x < t) {
      high = w - 1;
    } else if (x > t) {
      low = w + 1;
    } else {
      return w;
    }
  }
  return low;
}

async function numTasks(redis: Redis, protocol: string): Promise<number> {
  const key: string = `${protocol}:deposit_balance_verification_queue:0:queue`;
  const length: number | null = await retry(() => redis.client.llen(key));
  return (length != null) ? length : -1;
}

async function waitForKey(redis: Redis, key: string): Promise<void> {
  console.log(`Waiting for ${key} to appear`);
  while (true) {
    const data = await redis.client.exists(key);
    if (data !== null) {
      console.log(`Key ${key} is present`);
      return;
    }
    sleep(2000);
  }
}

async function waitProof(redis: Redis, key: string) {
  let needsChange: boolean = true;
  while (needsChange) {
    await sleep(8000);

    const value: string | null = await retry(() => redis.client.get(key));
    if (value == null) {
      console.log(`[I] waitProofs: value for ${key} does not yet exist.`);
      continue;
    }

    const proof: any = JSONbig.parse(value || "{}");
    if (proof.needsChange == null) {
      console.log(`[W] waitProofs: unexpected value for ${key}:`, value);
    } else {
      needsChange = Boolean(proof.needsChange);
      console.log(`[I] waitProofs: value for ${key} fetched, needsChange=${needsChange}`);
    }
  }
}

async function waitForSlot(currentSlot: bigint, referenceSlot: bigint): Promise<void> {
  const slotsToWait = Number(referenceSlot - currentSlot);
  if (slotsToWait > 0) {
    console.log(`Waiting for ${slotsToWait} slots until slot ${referenceSlot}`);
    await sleep(slotsToWait * 12000);
  }
}

async function waitForPubkeyCommitmentMapperProof(redis: Redis, protocol: string, blockNumber: number): Promise<void> {
  while (true) {
    const processingQueueKey = `${protocol}:pubkey_commitment_mapper:processing_queue`;
    const processingQueueHead = await redis.client.lindex(processingQueueKey, 0);

    const lastLoggedBlock = Number(await redis.client.get(`${protocol}:pubkey_commitment_mapper:last_logged_block`));
    console.log(`processingQueueHead: ${processingQueueHead}`);
    console.log(`lastLoggedBlock: ${lastLoggedBlock}`);
    console.log(`blockNumber: ${blockNumber}`);

    const blockHasBeenPassed = processingQueueHead === null
      ? lastLoggedBlock >= blockNumber
      : (() => {
        const headTaskBlockNumber = Number(processingQueueHead.split(',')[1]);
        console.log(`head task block number: ${headTaskBlockNumber}`);
        return lastLoggedBlock >= blockNumber && headTaskBlockNumber > blockNumber;
      })();

    if (blockHasBeenPassed) {
      console.log('pubkey commitment mapper proof found');
      return;
    }
    console.log('waiting for pubkey commitment mapper proof');

    await sleep(12_000);
  }
}

// use a different redis connection for validators commitment mapper
async function waitForValidatorsCommitmentMapperProof(redis: Redis, slot: number): Promise<void> {
  while (true) {
    const lastProcessedSlot = Number(await redis.client.get('last_processed_slot'));

    if (lastProcessedSlot >= slot) {
      const validatorsRootProofKey = `validator_proof:1`;
      const latestChangeSlot = await redis.getSlotWithLatestChange(validatorsRootProofKey, BigInt(slot));
      const validatorsRootKey = `validators_root:${latestChangeSlot}`;
      await waitForKey(redis, validatorsRootKey);

      return;
    }

    console.log(`waiting for last commitment mapper to catch up ${lastProcessedSlot}/${slot}`);

    await sleep(12_000);
  }
}
// +------+
// | Main |
// +------+

async function main() {
  const options = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .withBeaconNodeOpts()
    .option('json-rpc', {
      describe: 'The RPC URL',
      type: 'string',
      demandOption: true,
    })
    .option('address', {
      describe: 'The validators accumulator contract address',
      type: 'string',
      default: undefined,
    })
    .withRangeOpts()
    .withProtocolOpts()
    .option('snapshot-contract-address', {
      describe: 'The contract address',
      type: 'string',
      demandOption: true,
    })
    .withBeaconNodeOpts()
    .build();

  console.log('Running diva_balance_aggregator_scheduler:');
  console.log('\ttake:', options['take']);
  console.log('\toffset:', options['offset']);
  console.log('\tredis-host:', options['redis-host']);
  console.log('\tredis-port:', options['redis-port']);
  console.log('\tredis-auth:', options['redis-auth'].length);
  console.log('\taddress:', options['address']);
  console.log('\tjson-rpc:', options['json-rpc']);
  console.log('\tbeacon-node:', options['beacon-node']);
  console.log('\tprotocol:', options['protocol']);
  console.log('\tsnapshot-contract-address:', options['snapshot-contract-address']);
  console.log();

  const beaconApi = await getBeaconApi(options['beacon-node']);

  const redis: Redis = new Redis(
    options['redis-host'],
    options['redis-port'],
    options['redis-auth'],
  );

  const snapshotContractAddress = options['snapshot-contract-address'];
  const provider = new ethers.providers.JsonRpcProvider(options['json-rpc']);

  const snapshot = new ethers.Contract(
    snapshotContractAddress,
    accountManagerAbi,
    provider,
  );

  lightCleanQueue({
    redis: redis.client,
    protocol: options['protocol'],
    cleanDuration: 5000,
    silent: true,
  });

  console.log('Binding to SnapshotTaken events...');

  snapshot.on('SnapshotTaken', async (_: BigNumber, referenceSlot: BigNumber) => {
    const now: string = new Date().toISOString();
    console.log(`${now} | SnapshotTaken received: slot=${referenceSlot}`);

    await waitForSlot(await beaconApi.getHeadSlot(), BigInt(referenceSlot.toNumber()));
    await storeBalanceVerificationData({
      beaconNodeUrls: options['beacon-node'],
      slot: referenceSlot.toNumber(),
      take: options['take'],
      offset: options['offset'],
      redisHost: options['redis-host'],
      redisPort: options['redis-port'],
      redisAuth: options['redis-auth'],
      address: options['address'],
      rpcUrl: options['json-rpc'],
      protocol: options['protocol'],
    });

    const protocol: string = '' + options['protocol'];
    const tasks: number = await numTasks(redis, protocol);
    let instances: number = Math.min(MAX_INSTANCES, estimate(tasks));
    console.log(`[I] Running ${instances} worker(s) for ${tasks} task(s)...`);
    instances = 5;              // TODO
    let completed: number = 0;

    try {
      completed = await runTask(instances);
    } catch (e: unknown) {
      console.error(e);
    }

    if (completed === instances) {
      console.log(
        `[I] All workers have completed their tasks: instances=${instances}`,
      );
    } else {
      console.error(
        `[W] Some workers failed: completed=${completed} total=${instances}`,
      );
      // TODO: Handle error!
    }

    // get block number from slot
    const { beaconState } = await beaconApi.getBeaconState(BigInt(referenceSlot.toNumber()));
    const blockNumber = beaconState.latestExecutionPayloadHeader.blockNumber;

    // Wait for dependencies to get resolved before running the final layer
    await waitProof(redis, `${protocol}:${CONSTANTS.depositBalanceVerificationProofKey}:32:0`);
    await waitForPubkeyCommitmentMapperProof(redis, protocol, blockNumber);
    await waitForValidatorsCommitmentMapperProof(redis, referenceSlot.toNumber());

    console.log('[I] All proofs were committed!');
    // TODO: Launch next step!
  });
}

main().catch(console.error);
