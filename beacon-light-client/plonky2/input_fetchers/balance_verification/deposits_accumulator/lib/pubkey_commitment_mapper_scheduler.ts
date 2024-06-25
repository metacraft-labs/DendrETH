import { Redis } from '@dendreth/relay/implementations/redis';
import { formatHex } from '@dendreth/utils/ts-utils/bls';
import { sleep } from '@dendreth/utils/ts-utils/common-utils';
import { Contract, ethers } from 'ethers';
import { fetchEventsAsyncCB } from './event_fetcher';
import { ChainableCommander } from 'ioredis';
import { queryContractDeploymentBlockNumber } from './utils';

const sharedPrefix = `pubkey_commitment_mapper`;

const pubkeyProcessingQueueKey = `${sharedPrefix}:processing_queue`;
const pubkeysKey = `${sharedPrefix}:pubkeys`;
const currentlyComputedPubkeyMappingKey = `${sharedPrefix}:currently_computed_pubkey_mapping`;
const lastLoggedBlockKey = `${sharedPrefix}:last_logged_block`;

const depositEventName = 'Deposited';

interface SchedulerContext {
  redis: Redis;
  ethJsonRPC: ethers.providers.JsonRpcProvider;
  contract: Contract;
  protocol: string;
}

interface SchedulerContextConfig {
  redisHost: string;
  redisPort: number;
  redisAuth?: string;
  ethJsonRPCProviderURL: string;
  contractAddress: string;
  contractAbi: any;
  protocol: string;
}

export function createSchedulerContext(
  config: SchedulerContextConfig,
): SchedulerContext {
  const redis = new Redis(config.redisHost, config.redisPort, config.redisAuth);

  const ethJsonRPC = new ethers.providers.JsonRpcProvider(
    config.ethJsonRPCProviderURL,
  );

  const contract = new Contract(
    config.contractAddress,
    config.contractAbi,
    ethJsonRPC,
  );

  return {
    redis,
    ethJsonRPC,
    contract,
    protocol: config.protocol,
  };
}

export async function destroySchedulerContext(
  ctx: SchedulerContext,
): Promise<void> {
  await ctx.redis.quit();
}

export async function pollEvents(
  ctx: SchedulerContext,
  timeout: number = 12000,
): Promise<void> {
  const lastLoggedBlock = await ctx.redis.client.get(
    `${ctx.protocol}:${lastLoggedBlockKey}`,
  );
  if (lastLoggedBlock === null) {
    console.log('Error: Tree not initialized, use --rebuild');
    await destroySchedulerContext(ctx);
    process.exit(1);
  }

  while (true) {
    const lastLoggedBlock = await getLastLoggedBlock(ctx);
    const headBlockNumber = await ctx.ethJsonRPC.getBlockNumber();

    await fetchEventsAsyncCB(
      ctx.contract,
      depositEventName,
      lastLoggedBlock + 1,
      headBlockNumber,
      async event => {
        const args = event.args!;
        const pubkey = formatHex(args[0]);
        await updateDepositState(ctx, pubkey, event.blockNumber);
      },
    );

    await setLastLoggedBlock(ctx, headBlockNumber);
    await sleep(timeout);
  }
}

export async function rebuildCommitmentMapperTree(
  ctx: SchedulerContext,
): Promise<void> {
  const contractDeploymentBlockNumber =
    await queryContractDeploymentBlockNumber(
      ctx.ethJsonRPC,
      ctx.contract.address,
    );

  if (contractDeploymentBlockNumber === null) {
    console.log('Error: Invalid contract address');
    await destroySchedulerContext(ctx);
    process.exit(1);
  }

  await purgePubkeyCommitmentMapperData(ctx);

  await setCurrentlyComputedPubkeyMapping(ctx, 0);
  await setLastLoggedBlock(ctx, contractDeploymentBlockNumber - 1);

  const headBlockNumber = await ctx.ethJsonRPC.getBlockNumber();

  await fetchEventsAsyncCB(
    ctx.contract,
    depositEventName,
    contractDeploymentBlockNumber,
    headBlockNumber,
    async event => {
      const args = event.args!;
      const pubkey = formatHex(args[0]);
      await updateDepositState(ctx, pubkey, event.blockNumber);
    },
  );

  await setLastLoggedBlock(ctx, headBlockNumber);
}

async function updateDepositState(
  ctx: SchedulerContext,
  pubkey: string,
  blockNumber: number,
): Promise<void> {
  const pipeline = ctx.redis.client.multi();

  pushPubkeyToProcessingQueuePipe(pipeline, ctx.protocol, pubkey, blockNumber);
  pushPubkeyToRegistryPipe(pipeline, ctx.protocol, pubkey);
  setLastLoggedBlockPipe(pipeline, ctx.protocol, blockNumber);

  await pipeline.exec();

  console.log(`[${blockNumber}] ${pubkey}`);
}

export async function purgePubkeyCommitmentMapperData(
  ctx: SchedulerContext,
): Promise<void> {
  await ctx.redis.client.deletePattern(`${ctx.protocol}:${sharedPrefix}:*`);
}

async function setLastLoggedBlock(
  ctx: SchedulerContext,
  blockNumber: number,
): Promise<void> {
  await ctx.redis.client.set(
    `${ctx.protocol}:${lastLoggedBlockKey}`,
    blockNumber,
  );
}

async function getLastLoggedBlock(ctx: SchedulerContext): Promise<number> {
  const lastLoggedBlock = await ctx.redis.client.get(
    `${ctx.protocol}:${lastLoggedBlockKey}`,
  );
  return lastLoggedBlock ? Number(lastLoggedBlock) : 0;
}

async function setCurrentlyComputedPubkeyMapping(
  ctx: SchedulerContext,
  value: number,
): Promise<void> {
  await ctx.redis.client.set(
    `${ctx.protocol}:${currentlyComputedPubkeyMappingKey}`,
    value,
  );
}

function setLastLoggedBlockPipe(
  pipeline: ChainableCommander,
  protocol: string,
  blockNumber: number,
): void {
  pipeline.set(`${protocol}:${lastLoggedBlockKey}`, blockNumber);
}

function pushPubkeyToProcessingQueuePipe(
  pipeline: ChainableCommander,
  protocol: string,
  pubkey: string,
  blockNumber: number,
): void {
  pipeline.rpush(
    `${protocol}:${pubkeyProcessingQueueKey}`,
    `${pubkey},${blockNumber}`,
  );
}

function pushPubkeyToRegistryPipe(
  pipeline: ChainableCommander,
  protocol: string,
  pubkey: string,
): void {
  pipeline.rpush(`${protocol}:${pubkeysKey}`, pubkey);
}
