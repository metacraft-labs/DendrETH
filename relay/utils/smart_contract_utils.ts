import { IBeaconApi } from '../abstraction/beacon-api-interface';
import { ISmartContract } from '../abstraction/smart-contract-abstraction';
import { getGenericLogger } from '../../libs/typescript/ts-utils/logger';

const logger = getGenericLogger();

export async function getSlotOnChain(
  smartContract: ISmartContract,
  beaconApi: IBeaconApi,
) {
  const header_root_on_chain = await smartContract.optimisticHeaderRoot();

  logger.info(`header on chain ${header_root_on_chain}`);

  const lastSlotOnChain = await beaconApi.getBlockSlot(header_root_on_chain);
  return lastSlotOnChain;
}
