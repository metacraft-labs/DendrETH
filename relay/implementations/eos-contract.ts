import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import { formatHex } from '@dendreth/utils/ts-utils/bls';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';

const logger = getGenericLogger();

const exec = promisify(exec_);

export class EOSContract implements ISmartContract {
  private contractAddress: string;
  private rpcEndpoint: string;

  constructor(contractAddress: string, rpcEndpoint: string) {
    this.contractAddress = contractAddress;
    this.rpcEndpoint = rpcEndpoint;
  }

  async optimisticHeaderRoot(): Promise<string> {
    let queryCommand: string;
    let verifierTableKey = this.contractAddress;
    if (this.rpcEndpoint == 'local') {
      queryCommand = `cleos get table ${this.contractAddress} ${verifierTableKey} verifierdata`;
    } else {
      queryCommand = `cleos --url ${this.rpcEndpoint} get table ${this.contractAddress} ${verifierTableKey} verifierdata`;
    }

    const queryRes = JSON.parse((await exec(queryCommand)).stdout);
    const currentIndex = queryRes.rows[0].current_index;
    let lastHeader = queryRes.rows[0].new_optimistic_header_roots[currentIndex];
    logger.info(`lastHeader ${lastHeader}`);

    return '0x' + lastHeader;
  }

  async postUpdateOnChain(update: {
    attestedHeaderRoot: string;
    finalizedHeaderRoot: string;
    finalizedExecutionStateRoot: string;
    attestedHeaderSlot: number;
    a: string[];
    b: string[][];
    c: string[];
  }): Promise<any> {
    const updateData = JSON.stringify({
      key: this.contractAddress,
      proof_a: update.a.slice(0, 2).map(this.toHex),
      proof_b: [
        this.toHex(update.b[0][1]),
        this.toHex(update.b[0][0]),
        this.toHex(update.b[1][1]),
        this.toHex(update.b[1][0]),
      ],
      proof_c: update.c.slice(0, 2).map(this.toHex),
      new_optimistic_header_root: formatHex(update.attestedHeaderRoot),
      new_finalized_header_root: formatHex(update.finalizedHeaderRoot),
      new_execution_state_root: formatHex(update.finalizedExecutionStateRoot),
      new_slot: update.attestedHeaderSlot.toString(),
    });

    logger.info(`updating with data: ${updateData}`);
    let updateCommand: string;
    if (this.rpcEndpoint == 'local') {
      updateCommand = `cleos push action ${this.contractAddress} update '${updateData}' -p ${this.contractAddress}@active`;
    } else {
      updateCommand = `cleos --url ${this.rpcEndpoint} push action ${this.contractAddress} update '${updateData}' -p ${this.contractAddress}@active`;
    }
    logger.info(`updateCommand: ${updateCommand}`);
    let result = await exec(updateCommand);

    return result;
  }

  private toHex(number: string) {
    return BigInt(number).toString(16).padStart(64, '0');
  }
}
