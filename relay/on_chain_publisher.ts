import { GetUpdate, ProofResultType } from './types/types';
import { IBeaconApi } from './abstraction/beacon-api-interface';
import { IRedis } from './abstraction/redis-interface';
import { ISmartContract } from './abstraction/smart-contract-abstraction';
import { Contract } from 'ethers';
import {
  TransactionSpeed,
  getSolidityProof,
  publishTransaction,
} from './implementations/publish_evm_transaction';
import Web3 from 'web3';
import { checkConfig, sleep } from '../libs/typescript/ts-utils/common-utils';
import { Queue } from 'bullmq';
import { Config, UPDATE_POLING_QUEUE } from './constants/constants';
import crypto from 'crypto';
import { getSlotOnChain } from './utils/smart_contract_utils';
import { addUpdate } from './utils/orchestrator';

let isDrainRunning = false;

export async function publishProofs(
  redis: IRedis,
  beaconApi: IBeaconApi,
  smartContract: ISmartContract,
  networkConfig: Config,
  slotsJump: number,
  hashiAdapterContract?: Contract | undefined,
  rpcEndpoint?: string,
  transactionSpeed: TransactionSpeed = 'avg',
) {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(config);

  askForUpdates(config, smartContract, beaconApi, slotsJump, networkConfig);

  try {
    await drainUpdatesInRedis(
      redis,
      beaconApi,
      smartContract,
      hashiAdapterContract,
      rpcEndpoint,
      transactionSpeed,
    );

    await redis.subscribeForProofs(async () => {
      try {
        await drainUpdatesInRedis(
          redis,
          beaconApi,
          smartContract,
          hashiAdapterContract,
          rpcEndpoint,
          transactionSpeed,
        );
      } catch (e) {
        console.error('Error while draining updates in Redis', e);
      }
    });
  } catch (error) {
    console.error('Error occurred while publishing proofs: ', error);
    throw error;
  }
}

export async function drainUpdatesInRedis(
  redis: IRedis,
  beaconApi: IBeaconApi,
  smartContract: ISmartContract,
  hashiAdapterContract: Contract | undefined,
  rpcEndpoint?: string,
  transactionSpeed: TransactionSpeed = 'avg',
) {
  if (isDrainRunning) {
    console.log('Publishing transactions is already running');
    return;
  }
  isDrainRunning = true;
  let failedNumber = 0;
  while (true) {
    try {
      const lastSlotOnChain = await getSlotOnChain(smartContract, beaconApi);

      const proofResult = await redis.getNextProof(lastSlotOnChain);

      if (proofResult == null) {
        console.log('No proof to publish');
        isDrainRunning = false;
        return;
      }

      try {
        await postUpdateOnChain(
          proofResult,
          smartContract,
          beaconApi,
          lastSlotOnChain,
          hashiAdapterContract,
          rpcEndpoint,
          transactionSpeed,
        );
        // Slow down broadcasting
        await sleep(2000);
        failedNumber = 0;
      } catch (error) {
        [failedNumber, isDrainRunning] = (await handleFailure(
          error,
          'publishing transaction',
          failedNumber,
        )) as any[];
      }
    } catch (error) {
      [failedNumber, isDrainRunning] = (await handleFailure(
        error,
        'fetching header',
        failedNumber,
      )) as any[];
    }
  }
}

export async function postUpdateOnChain(
  proofResult: ProofResultType,
  lightClientContract: ISmartContract,
  beaconApi: IBeaconApi,
  lastSlotOnChain: number,
  hashiAdapterContract: Contract | undefined,
  rpcEndpoint?: string,
  transactionSpeed: TransactionSpeed = 'avg',
) {
  const update = {
    attestedHeaderRoot:
      '0x' +
      BigInt('0b' + proofResult.proofInput.nextHeaderHash.join(''))
        .toString(16)
        .padStart(64, '0'),
    attestedHeaderSlot: proofResult.proofInput.nextHeaderSlot,
    finalizedHeaderRoot:
      '0x' +
      BigInt('0b' + proofResult.proofInput.finalizedHeaderRoot.join(''))
        .toString(16)
        .padStart(64, '0'),
    finalizedExecutionStateRoot:
      '0x' +
      BigInt('0b' + proofResult.proofInput.execution_state_root.join(''))
        .toString(16)
        .padStart(64, '0'),
  };

  console.log(update);

  if (hashiAdapterContract) {
    const finalizedHeaderSlot = await beaconApi.getBlockSlot(
      '0x' +
        BigInt(
          '0b' +
            proofResult.proofInput.finalizedHeaderRoot
              .join('')
              .padStart(64, '0'),
        ).toString(16),
    );

    const hashiInfo = await beaconApi.getHashiAdapterInfo(finalizedHeaderSlot);

    const solidityProof = await getSolidityProof({
      a: proofResult.proof.pi_a,
      b: proofResult.proof.pi_b,
      c: proofResult.proof.pi_c,
    });

    await publishTransaction(
      hashiAdapterContract,
      'storeBlockHeader(uint32,uint64,bytes32[],uint256,bytes32[],bytes32,bytes32[],(bytes32,uint256,bytes32,bytes32,uint256[2],uint256[2][2],uint256[2]))',
      [
        (await hashiAdapterContract.provider.getNetwork()).chainId,
        finalizedHeaderSlot,
        hashiInfo.slotProof.map(x => '0x' + x),
        hashiInfo.blockNumber,
        hashiInfo.blockNumberProof.map(x => '0x' + x),
        '0x' + hashiInfo.blockHash,
        hashiInfo.blockHashProof.map(x => '0x' + x),
        { ...update, ...solidityProof },
      ],
      new Web3(rpcEndpoint!),
      transactionSpeed,
      true,
    );
  } else {
    await lightClientContract.postUpdateOnChain({
      ...update,
      a: proofResult.proof.pi_a,
      b: proofResult.proof.pi_b,
      c: proofResult.proof.pi_c,
    });
  }

  const transactionSlot = proofResult.proofInput.nextHeaderSlot;

  const currentHeadSlot = await beaconApi.getCurrentHeadSlot();

  console.log(`Previous slot on the chain ${lastSlotOnChain}`);

  console.log(`Transaction publishing for slot ${transactionSlot}`);

  console.log(`Current slot on the network is ${currentHeadSlot}`);

  console.log(
    `Prev slot is ${
      ((currentHeadSlot - lastSlotOnChain) * 12) / 60
    } minutes behind`,
  );

  console.log(
    `Transaction is ${
      ((currentHeadSlot - transactionSlot) * 12) / 60
    } minutes behind`,
  );
}

async function handleFailure(
  error: any,
  scopeError: string,
  failedNumber: number,
): Promise<[number, boolean]> {
  if (failedNumber > 10) {
    log(error, `ERROR occurred in ${scopeError}`, 'STOPPING');
    isDrainRunning = false;
    return [failedNumber, isDrainRunning];
  }
  log(error, `ERROR occurred in ${scopeError}`, 'will retry');
  failedNumber++;
  await sleep(10000);

  return [failedNumber, isDrainRunning];
}

function log(error: any, firstMessage: string, secondMessage: string): void {
  console.log(firstMessage);
  console.log(error);
  console.log(secondMessage);
}

async function askForUpdates(
  config: { REDIS_HOST: string | undefined; REDIS_PORT: number },
  smartContract: ISmartContract,
  beaconApi: IBeaconApi,
  slotsJump: number,
  networkConfig: Config,
) {
  const updateQueue = new Queue<GetUpdate>(UPDATE_POLING_QUEUE, {
    connection: {
      host: config.REDIS_HOST!,
      port: Number(config.REDIS_PORT),
    },
  });

  while (true) {
    try {
      let before = Date.now();

      const optimisticSlot = await getSlotOnChain(smartContract, beaconApi);
      const headSlot = await beaconApi.getCurrentHeadSlot();

      // Loop through
      while (
        await addUpdate(
          optimisticSlot,
          slotsJump,
          headSlot,
          updateQueue,
          networkConfig,
          beaconApi,
        )
      ) {}

      let after = Date.now();
      await sleep(12000 * slotsJump - (after - before));
    } catch (e) {
      console.error('Error while fetching update', e);
    }
  }
}
