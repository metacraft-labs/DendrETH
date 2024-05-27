import { Redis } from '@dendreth/relay/implementations/redis';
import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import { ethers } from 'ethers';
import { getEvents } from './event-fetcher';
import ValidatorsAccumulator from '../../../../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';
import chalk from 'chalk';

enum Events {
  Deposited = 'Deposited',
}

export abstract class Scheduler {
  protected redis: Redis;
  protected queue: any;
  protected provider: ethers.providers.JsonRpcProvider;
  protected contract: ethers.Contract;
  protected depositsCount: number;
  protected syncBlock: number;
  protected ssz: any;

  async init(
    options: any,
    constants: {
      queue: string;
      latestBlock: string;
    },
  ) {
    this.redis = new Redis(options['redis-host'], options['redis-port']);
    this.queue = new WorkQueue(new KeyPrefix(`${constants.queue}`));
    this.provider = new ethers.providers.JsonRpcProvider(options['rpc-url']);
    this.contract = new ethers.Contract(
      options['address'],
      ValidatorsAccumulator.abi,
      this.provider,
    );

    let latestLoggedBlock = await this.redis.get(constants.latestBlock);
    if (!latestLoggedBlock) {
      latestLoggedBlock = (
        (await this.provider.getBlockNumber()) - 100000
      ).toString();
    }
    this.syncBlock = options['sync-block'] || +latestLoggedBlock;

    this.depositsCount = await this.redis.getDepositsCount();

    const mod = await import('@lodestar/types');
    this.ssz = mod.ssz;
  }

  async dispose() {
    return this.redis.quit();
  }

  async start() {
    // Sync to latest block
    await this.syncToLatestBlock();

    // Start listening to head events
    this.contract.on(Events.Deposited, (...args: any) => {
      const data = args[args.length - 1];
      if (data.blockNumber < this.syncBlock) {
        return;
      }
      this.updateDeposits(
        {
          pubkey: args[0],
          signature: args[1],
          depositMessageRoot: args[2],
          index: args[3],
        },
        data.blockNumber,
      );
    });
  }

  async syncToLatestBlock() {
    const latestBlock = await this.provider.getBlockNumber();
    console.log(
      chalk.bold.blue(`Syncing to block (${chalk.cyan(latestBlock)}...`),
    );

    if (this.syncBlock > latestBlock) {
      return;
    }

    const logs = await getEvents(
      this.provider,
      this.contract,
      {
        [Events.Deposited]: [
          'pubkey',
          'depositIndex',
          'signature',
          'depositMessageRoot',
        ],
      },
      this.syncBlock,
      latestBlock,
    );

    for (const log of logs) {
      await this.updateDeposits(
        log['Deposited'] as any,
        log['Deposited'].blockNumber,
      );
    }
  }

  abstract updateDeposits(
    event: {
      pubkey: string;
      signature: string;
      depositMessageRoot: string;
      index: string;
    },
    blockNumber: number,
  ): Promise<void>;
}
