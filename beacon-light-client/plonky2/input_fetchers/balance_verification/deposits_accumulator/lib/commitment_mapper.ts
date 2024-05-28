import { Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../../../../kv_db_constants.json';
import { Scheduler } from './scheduler';
import chalk from 'chalk';
import { makeBranchIterator } from '@dendreth/utils/ts-utils/common-utils';

enum TaskTag {
  PROVE_ZERO_FOR_DEPTH = 0,
  DEPOSIT_PROOF = 1,
  DEPOSIT_NODE_PROOF = 2,
}

export class CommitmentMapperScheduler extends Scheduler {
  protected depositIndices: string[] = [];

  async init(options: any) {
    await super.init(options, {
      queue: CONSTANTS.depositQueue,
      latestBlock: CONSTANTS.depositLatestBlockKey,
    });

    this.depositIndices = await this.redis.getDepositsIndices();
  }

  async start() {
    if (await this.redis.isZeroDepositsEmpty()) {
      console.log(chalk.bold.blue('Adding zero tasks...'));
      await this.scheduleZeroTasks();
    }

    await super.start();
  }

  override async updateDepositsBatch(
    deposits: {
      event: {
        pubkey: string;
        signature: string;
        depositMessageRoot: string;
        depositIndex: string;
      };
      blockNumber: number;
    }[],
  ): Promise<void> {
    deposits = deposits.filter(
      d => !this.depositIndices.includes(d.event.depositIndex),
    );
    if (deposits.length === 0) {
      return;
    }

    if (deposits.length === 1) {
      console.log(
        chalk.blue(
          `Processing deposit for ${chalk.yellow(
            deposits[0].event.pubkey.slice(0, 6),
          )}...${chalk.yellow(deposits[0].event.pubkey.slice(-4))}`,
        ),
      );
    } else {
      console.log(chalk.bold.blue(`Processing ${deposits.length} deposits...`));
    }

    const depositsWithIndices = await Promise.all(
      deposits.map(async deposit => {
        this.depositIndices.push(deposit.event.depositIndex);
        const depositIndex = this.depositIndices.length - 1;

        await this.scheduleDepositProof(BigInt(depositIndex));

        return {
          index: depositIndex,
          data: {
            deposit: deposit.event,
            isReal: true,
          },
        };
      }),
    );

    await this.redis.saveDeposits(depositsWithIndices);

    await this.updateBranches(depositsWithIndices.map(d => d.index));

    const blockNumber = deposits[deposits.length - 1].blockNumber;
    await this.redis.set(CONSTANTS.depositLatestBlockKey, `${blockNumber + 1}`);

    console.log(
      chalk.bold.green(
        `${deposits.length} deposit${
          deposits.length > 1 ? 's' : ''
        } added (${chalk.yellow(blockNumber)} block)`,
      ),
    );
  }

  public async updateBranches(depositIndices: number[]) {
    let levelIterator = makeBranchIterator(depositIndices.map(BigInt), 40n);

    let leafs = levelIterator.next().value!;

    await Promise.all(leafs.map(gindex => this.redis.saveDepositProof(gindex)));

    for (const gindices of levelIterator) {
      await Promise.all(
        gindices.map(gindex => this.redis.saveDepositProof(gindex)),
      );

      await Promise.all(
        gindices.map(gindex => this.scheduleUpdateProofNodeTask(gindex)),
      );
    }
  }

  async scheduleZeroTasks() {
    await this.redis.saveDeposits([
      {
        index: Number(CONSTANTS.validatorRegistryLimit),
        data: {
          deposit: {
            pubkey: ''.padEnd(96, '0'),
            signature: ''.padEnd(192, '0'),
            depositIndex: '0',
            depositMessageRoot: ''.padEnd(64, '0'),
          },
          isReal: false,
        },
      },
    ]);

    await this.scheduleDepositProof(BigInt(CONSTANTS.validatorRegistryLimit));
    await this.redis.saveDummyDepositProof(40n);

    for (let depth = 39n; depth >= 0n; depth--) {
      this.scheduleProveZeroForDepth(depth);
      await this.redis.saveDummyDepositProof(depth);
    }
  }

  async scheduleProveZeroForDepth(depth: bigint) {
    await this.scheduleTask(TaskTag.PROVE_ZERO_FOR_DEPTH, depth);
  }

  async scheduleUpdateProofNodeTask(gindex: bigint) {
    await this.scheduleTask(TaskTag.DEPOSIT_NODE_PROOF, gindex);
  }

  async scheduleDepositProof(index: bigint) {
    await this.scheduleTask(TaskTag.DEPOSIT_PROOF, index);
  }

  async scheduleTask(task: TaskTag, value: bigint) {
    const buffer = new ArrayBuffer(9);
    const dataView = new DataView(buffer);

    dataView.setUint8(0, task);
    dataView.setBigUint64(1, value, false);

    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));
  }
}
