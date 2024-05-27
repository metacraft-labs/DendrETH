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

  async init(options: any) {
    super.init(options, {
      queue: CONSTANTS.depositSignatureVerificationQueue,
      latestBlock: CONSTANTS.depositSignatureVerificationLatestBlockKey,
    });
    this.api = await getBeaconApi(options['beacon-node']);

    this.GENESIS_FORK_VERSION = ethers.utils.hexlify(
      (await this.api.getGenesisData()).genesisForkVersion,
    );
  }

  override async updateDeposits(
    event: {
      pubkey: string;
      signature: string;
      depositMessageRoot: string;
      index: string;
    },
    blockNumber: number,
  ): Promise<void> {
    console.log(
      chalk.blue(`Processing deposit for ${chalk.bold.yellow(event.pubkey)}`),
    );
    const fork_data_root = bytesToHex(
      this.ssz.phase0.ForkData.hashTreeRoot({
        currentVersion: hexToBytes(this.GENESIS_FORK_VERSION),
        genesisValidatorsRoot: hexToBytes(GENESIS_VALIDATOR_ROOT),
      }),
    );

    const domain =
      formatHex(DOMAIN_DEPOSIT) + formatHex(fork_data_root.slice(0, 56));

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

    await this.redis.set(
      CONSTANTS.depositSignatureVerificationQueue,
      `${blockNumber + 1}`,
    );
    console.log(
      chalk.bold.green(
        `Deposit ${this.depositsCount} added (${chalk.yellow(
          blockNumber,
        )} block)`,
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
