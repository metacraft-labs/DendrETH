import { Redis } from '@dendreth/relay/implementations/redis';
import { Item, KeyPrefix, WorkQueue } from '@mevitae/redis-work-queue';
import CONSTANTS from '../constants/validator_commitment_constants.json';
import { ethers } from 'ethers';
import { getEvents } from './event-fetcher';
import ValidatorsAccumulator from '../../solidity/artifacts/contracts/validators_accumulator/ValidatorsAccumulator.sol/ValidatorsAccumulator.json';
import { hexToBytes } from '@dendreth/utils/ts-utils/bls';

enum Events {
  Deposited = 'Deposited',
}
const DOMAIN =
  '03000000f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a9';

export class DepositScheduler {
  private redis: Redis;
  private queue: any;
  private provider: ethers.providers.JsonRpcProvider;
  private contract: ethers.Contract;
  private depositsCount: number;
  private syncBlock: number;
  private ssz: any;

  async init(options: any) {
    this.redis = new Redis(options['redis-host'], options['redis-port']);
    this.queue = new WorkQueue(new KeyPrefix(`${CONSTANTS.depositsQueue}`));
    this.provider = new ethers.providers.JsonRpcProvider(options['rpc-url']);
    this.contract = new ethers.Contract(
      options['address'],
      ValidatorsAccumulator.abi,
      this.provider,
    );

    let latestLoggedBlock = await this.redis.get(
      CONSTANTS.latestLoggedBlockKey,
    );
    if (!latestLoggedBlock) {
      latestLoggedBlock = (
        (await this.provider.getBlockNumber()) - 100000
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
    this.depositsCount = await this.redis.getDepositsCount();

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
    const toBlock = latestBlock;
    if (this.syncBlock > latestBlock) {
      return;
    }

    const logs = await getEvents(
      this.provider,
      this.contract,
      {
        [Events.Deposited]: [
          'pubkey',
          'signature',
          'depositMessageRoot',
          'depositIndex',
        ],
      },
      this.syncBlock,
      toBlock,
    );

    for (const log of logs) {
      await this.updateDeposits(
        log['Deposited'] as any,
        log['Deposited'].blockNumber,
      );
    }
  }

  async updateDeposits(
    event: {
      pubkey: string;
      signature: string;
      depositMessageRoot: string;
      index: string;
    },
    blockNumber: number,
  ) {
    const signing_root = this.ssz.phase0.SigningData.hashTreeRoot({
      objectRoot: hexToBytes(event.depositMessageRoot),
      domain: hexToBytes(DOMAIN),
    });

    await this.redis.saveDeposit(this.depositsCount, {
      pubkey: event.pubkey,
      signature: event.signature,
      signingRoot: ethers.utils.hexlify(signing_root),
    });

    await this.scheduleDepositSignatureProof(BigInt(this.depositsCount));

    this.depositsCount++;

    await this.redis.set(CONSTANTS.latestLoggedBlockKey, `${blockNumber + 1}`);
  }

  async scheduleDepositSignatureProof(index: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);
    dataView.setUint8(0, 0);
    dataView.setBigUint64(1, index, false);
    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));
  }
}
