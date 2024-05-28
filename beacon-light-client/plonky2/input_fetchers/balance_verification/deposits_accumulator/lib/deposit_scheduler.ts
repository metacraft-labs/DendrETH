import { Item } from '@mevitae/redis-work-queue';
import CONSTANTS from '../../../../kv_db_constants.json';
import { ethers } from 'ethers';
import {
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { Scheduler } from './scheduler';
import chalk from 'chalk';

const DOMAIN_DEPOSIT = '0x03000000';
const GENESIS_VALIDATOR_ROOT =
  '0x0000000000000000000000000000000000000000000000000000000000000000';

export class DepositScheduler extends Scheduler {
  private api: BeaconApi;
  private GENESIS_FORK_VERSION: string;
  protected depositsCount: number;

  async init(options: any) {
    await super.init(options, {
      queue: CONSTANTS.depositSignatureVerificationQueue,
      latestBlock: CONSTANTS.depositSignatureVerificationLatestBlockKey,
    });
    this.api = await getBeaconApi(options['beacon-node']);
    this.depositsCount = await this.redis.getDepositsCount();

    this.GENESIS_FORK_VERSION = ethers.utils.hexlify(
      (await this.api.getGenesisData()).genesisForkVersion,
    );
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

    const fork_data_root = bytesToHex(
      this.ssz.phase0.ForkData.hashTreeRoot({
        currentVersion: hexToBytes(this.GENESIS_FORK_VERSION),
        genesisValidatorsRoot: hexToBytes(GENESIS_VALIDATOR_ROOT),
      }),
    );

    const domain =
      formatHex(DOMAIN_DEPOSIT) + formatHex(fork_data_root.slice(0, 56));

    for (const { event } of deposits) {
      const signing_root = this.ssz.phase0.SigningData.hashTreeRoot({
        objectRoot: hexToBytes(event.depositMessageRoot),
        domain: hexToBytes(domain),
      });

      await this.redis.saveDepositSignatureVerification(this.depositsCount, {
        pubkey: event.pubkey,
        signature: event.signature,
        signingRoot: ethers.utils.hexlify(signing_root),
      });

      await this.scheduleDepositSignatureProof(BigInt(this.depositsCount));

      this.depositsCount++;
    }

    const blockNumber = deposits[deposits.length - 1].blockNumber;
    await this.redis.set(
      CONSTANTS.depositSignatureVerificationLatestBlockKey,
      `${blockNumber + 1}`,
    );
    console.log(
      chalk.bold.green(
        `${deposits.length} deposit${
          deposits.length > 1 ? 's' : ''
        } added (${chalk.yellow(blockNumber)} block)`,
      ),
    );
  }

  async scheduleDepositSignatureProof(index: bigint) {
    const buffer = new ArrayBuffer(8);
    const dataView = new DataView(buffer);

    dataView.setBigUint64(0, index, false);

    this.queue.addItem(this.redis.client, new Item(Buffer.from(buffer)));
  }
}
