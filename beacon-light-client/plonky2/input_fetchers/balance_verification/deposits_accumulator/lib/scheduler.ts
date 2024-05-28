import { Redis } from '@dendreth/relay/implementations/redis';
import { KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import { ethers } from 'ethers';
import { getEvents } from './event-fetcher';
import ValidatorsAccumulator from '../../../../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';
import chalk from 'chalk';

enum Events {
  Deposited = 'Deposited',
}
const EVENTS_BATCH_SIZE = 10_000;

export abstract class Scheduler {
  protected redis: Redis;
  protected queue: any;
  protected provider: ethers.providers.JsonRpcProvider;
  protected contract: ethers.Contract;
  protected syncBlock: number;
  protected ssz: any;

  async init(
    options: any,
    constants: {
      queue: string;
      latestBlock: string;
    },
  ): Promise<void> {
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
        (await this.provider.getBlockNumber()) - EVENTS_BATCH_SIZE
      ).toString();
    }
    this.syncBlock = options['sync-block'] || +latestLoggedBlock;

    const mod = await import('@lodestar/types');
    this.ssz = mod.ssz;
  }

  async dispose() {
    return this.redis.quit();
  }

  async start() {
    // Sync to latest block
    await this.syncToLatestBlock();

    console.log(chalk.bold.blue('Listening for events...'));
    // Start listening to head events
    this.contract.on(Events.Deposited, async (...args: any) => {
      const data = args[args.length - 1];

      if (this.syncBlock <= data.blockNumber) {
        await this.updateDepositsBatch([
          {
            event: {
              pubkey: data.args.pubkey,
              depositIndex: data.args.depositIndex,
              signature: data.args.signature,
              depositMessageRoot: data.args.depositMessageRoot,
            },
            blockNumber: data.blockNumber,
          },
        ]);
      }
    });
  }

  async syncToLatestBlock() {
    const latestBlock = await this.provider.getBlockNumber();
    console.log(
      chalk.bold.blue(`Syncing to block (${chalk.cyan(latestBlock)})...`),
    );

    if (this.syncBlock > latestBlock) {
      return;
    }

    const iterations = Math.floor(
      (latestBlock - this.syncBlock) / EVENTS_BATCH_SIZE,
    );

    let deposits: any[] = [];
    for (let i = 0; i < iterations; i++) {
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
        this.syncBlock + EVENTS_BATCH_SIZE - 1,
      );

      deposits.push(
        ...logs.map(log => {
          return {
            event: log[Events.Deposited],
            blockNumber: log[Events.Deposited].blockNumber,
          };
        }),
      );

      this.syncBlock += EVENTS_BATCH_SIZE;
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

    deposits.push(
      ...logs.map(log => {
        return {
          event: log[Events.Deposited],
          blockNumber: log[Events.Deposited].blockNumber,
        };
      }),
    );

    this.syncBlock = latestBlock + 1;

    await this.updateDepositsBatch(deposits);
  }

  abstract updateDepositsBatch(
    deposits: {
      event: {
        pubkey: string;
        signature: string;
        depositMessageRoot: string;
        depositIndex: string;
      };
      blockNumber: number;
    }[],
  ): Promise<void>;
}
