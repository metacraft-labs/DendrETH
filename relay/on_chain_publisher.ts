import { groth16 } from 'snarkjs';
import { ProofResultType } from './types/types';
import { IBeaconApi } from './abstraction/beacon-api-interface';
import { IRedis } from './abstraction/redis-interface';
import { ISmartContract } from './abstraction/smart-contract-abstraction';

export async function publishProofs(
  redis: IRedis,
  beaconApi: IBeaconApi,
  smartContract: ISmartContract,
) {
  await drainUpdatesInRedis(redis, beaconApi, smartContract);

  await redis.subscribeForProofs(async () => {
    try {
      drainUpdatesInRedis(redis, beaconApi, smartContract);
    } catch (e) {
      console.log('error happened');
      console.log(e);
    }
  });
}

export async function drainUpdatesInRedis(
  redis: IRedis,
  beaconApi: IBeaconApi,
  smartContract: ISmartContract,
) {
  let failedNumber = 0;
  while (true) {
    try {
      const header_root_on_chain = await smartContract.optimisticHeaderRoot();

      console.log(header_root_on_chain);

      const lastSlotOnChain = await beaconApi.getBlockSlot(
        header_root_on_chain,
      );

      const proofResult = await redis.getNextProof(lastSlotOnChain);

      if (proofResult == null) {
        return;
      }

      try {
        await postUpdateOnChain(proofResult, smartContract);
        // Slow down broadcasting
        await new Promise(r => setTimeout(r, 2000));
        failedNumber = 0;
      } catch (error) {
        if (failedNumber > 10) {
          console.log('ERROR occured in publishing transaction');
          console.log(error);
          console.log('STOPPING');
          return;
        }
        console.log('ERROR occured in publishing transaction');
        console.log(error);
        console.log('will retry');
        failedNumber++;
        await new Promise(r => setTimeout(r, 10000));
      }
    } catch (error) {
      if (failedNumber > 10) {
        console.log('error occured while fetching header');
        console.log(error);
        console.log('STOPPING');
        return;
      }

      console.log('error occured while fetching header');
      console.log(error);
      console.log('will retry');
      failedNumber++;
      await new Promise(r => setTimeout(r, 10000));
    }
  }
}

export async function postUpdateOnChain(
  proofResult: ProofResultType,
  lightClientContract: ISmartContract,
) {
  const calldata = await groth16.exportSolidityCallData(
    proofResult.proof,
    proofResult.proof.public,
  );

  const argv: string[] = calldata
    .replace(/["[\]\s]/g, '')
    .split(',')
    .map(x => BigInt(x).toString());

  const a = [argv[0], argv[1]];
  const b = [
    [argv[2], argv[3]],
    [argv[4], argv[5]],
  ];
  const c = [argv[6], argv[7]];

  const transaction = await lightClientContract.postUpdateOnChain({
    attested_header_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.nextHeaderHash.join(''))
        .toString(16)
        .padStart(64, '0'),
    finalized_header_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.finalizedHeaderRoot.join(''))
        .toString(16)
        .padStart(64, '0'),
    finalized_execution_state_root:
      '0x' +
      BigInt('0b' + proofResult.proofInput.execution_state_root.join(''))
        .toString(16)
        .padStart(64, '0'),
    a: a,
    b: b,
    c: c,
  });

  console.log(transaction);

  await transaction.wait();
}
