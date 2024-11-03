import { task } from 'hardhat/config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import Web3 from 'web3';
import { publishTransaction } from '@dendreth/relay/implementations/publish_evm_transaction';

const logger = getGenericLogger();

task('change-adapter-address', 'Verify')
  .addParam('lightClient', 'The address of the BeaconLightClient contract')
  .addParam('adapter', 'The address of the adapter contract')
  .addParam(
    'transactionSpeed',
    'The speed you want the transactions to be included in a block',
    'avg',
    undefined,
    true,
  )
  .setAction(async (args, { network, ethers }) => {
    const [publisher] = await ethers.getSigners();

    logger.info(`Changing adapter with address ${publisher.address}`);

    const lightClientContract = await ethers.getContractAt(
      'BeaconLightClient',
      args.lightClient,
      publisher,
    );

    const web3 = new Web3((network.config as any).url);

    await publishTransaction(
      lightClientContract,
      'changeAdapterAddress',
      [args.adapter],
      web3,
      args.transactionSpeed,
      true,
    );

    logger.info(`Adapter changed to ${args.adapter}`);
  });
