import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import { compileVerifierParseDataTool } from '../../tests/helpers/helpers';
import { getDataFromPrintHeaderResult } from '../../libs/typescript/cosmos-utils/cosmos-utils';

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
      queryCommand = `cleos push action ${this.contractAddress} printheader '{\"key\":\"${verifierTableKey}\"}' -p ${this.contractAddress}@active`;
    } else {
      queryCommand = `cleos --url ${this.rpcEndpoint} push action ${this.contractAddress} printheader '{\"key\":\"${verifierTableKey}\"}' -p ${this.contractAddress}@active`;
    }
    const queryRes = await exec(queryCommand);
    let lastHeader = getDataFromPrintHeaderResult((await queryRes).stdout);

    return lastHeader;
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
    const parseDataTool = await compileVerifierParseDataTool('eos', 'verifier');

    const flattedB = update.b.flat();
    const parseUpdateDataCommand = `${parseDataTool} updateDataForEOSContractClass \
  --attested_header_rootEOS=${
    update.attestedHeaderRoot
  } --finalized_header_rootEOS=${
      update.finalizedHeaderRoot
    } --finalized_execution_state_rootEOS= ${
      update.finalizedExecutionStateRoot
    } \
  --aEOS=${update.a[0]} --aEOS=${update.a[1]} --aEOS=${update.a[2]} \
  --bEOS=${flattedB[0]} --bEOS=${flattedB[1]} --bEOS=${flattedB[2]} --bEOS=${
      flattedB[3]
    } --bEOS=${flattedB[4]} --bEOS=${flattedB[5]} \
  --cEOS=${update.c[0]} --cEOS=${update.c[1]} --cEOS=${
      update.c[2]
    } --attested_header_slotEOS=${update.attestedHeaderSlot.toString()}`;
    const updateDataExec = exec(parseUpdateDataCommand);
    const updateData = (await updateDataExec).stdout.replace(/\s/g, '');
    console.info('updating with data:', updateData);
    let updateCommand: string;
    if (this.rpcEndpoint == 'local') {
      updateCommand = `cleos push action ${this.contractAddress} update ${updateData} -p ${this.contractAddress}@active`;
    } else {
      updateCommand = `cleos --url ${this.rpcEndpoint} push action ${this.contractAddress} update ${updateData} -p ${this.contractAddress}@active`;
    }
    console.info('updateCommand:', updateCommand);
    let result = await exec(updateCommand);

    return result;
  }
}
