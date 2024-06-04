import { task } from 'hardhat/config';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { SolidityContract } from '@dendreth/relay/implementations/solidity-contract';
import { publishProofs } from '@dendreth/relay/on_chain_publisher';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Contract, ethers } from 'ethers';
import hashi_abi from './hashi_abi.json';
import { getNetworkConfig } from '@dendreth/relay/utils/get_current_network_config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { initPrometheusSetup } from '@dendreth/utils/ts-utils/prometheus-utils';

const logger = getGenericLogger();

task('start-publishing', 'Run relayer')
  .addParam('lightClient', 'The address of the BeaconLightClient contract')
  .addParam('followNetwork', 'The network the contract follows')
  .addParam(
    'privateKey',
    'The private key that will be used to publish',
    undefined,
    undefined,
    true,
  )
  .addParam(
    'transactionSpeed',
    'The speed you want the transactions to be included in a block',
    'avg',
    undefined,
    true,
  )
  .addParam('slotsJump', 'The number of slots to jump')
  .addParam(
    'hashi',
    'The address of the Hashi adapter contract',
    '',
    undefined,
    true,
  )
  .addParam(
    'prometheusPort',
    'Port No. (3000-3005) for Node Express server where Prometheus is listening.',
    '',
    undefined,
    true,
  )
  .setAction(async (args, { ethers, network }) => {
    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    if (args.followNetwork !== 'pratter' && args.followNetwork !== 'mainnet') {
      logger.warn('This followNetwork is not specified in networkconfig');
      return;
    }

    if (args.prometheusPort) {
      console.log(`Initializing Prometheus on port ${args.prometheusPort}`);

      let networkName: string = '';
      for (let i = 0; i < process.argv.length; i++) {
        const arg = process.argv[i];
        if (arg === '--network' && i + 1 < process.argv.length) {
          networkName = process.argv[i + 1];
          break;
        }
      }

      initPrometheusSetup(args.prometheusPort, networkName);
    }

    const currentConfig = await getNetworkConfig(args.followNetwork);

    let publisher;

    if (!args.privateKey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privateKey, ethers.provider);
    }

    logger.info(`Publishing updates with the account: ${publisher.address}`);
    logger.info(
      `Account balance: ${(await publisher.getBalance()).toString()}`,
    );

    logger.info(`Contract address ${args.lightClient}`);

    const lightClientContract = await ethers.getContractAt(
      'BeaconLightClient',
      args.lightClient,
      publisher,
    );

    if (
      args.transactionSpeed &&
      !['slow', 'avg', 'fast'].includes(args.transactionSpeed)
    ) {
      throw new Error('Invalid transaction speed');
    }

    let hashiAdapterContract: ethers.Contract | undefined;

    if (args.hashi) {
      hashiAdapterContract = new Contract(args.hashi, hashi_abi, publisher);
    }

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    const beaconApi = await getBeaconApi(currentConfig.BEACON_REST_API);
    const contract = new SolidityContract(
      lightClientContract,
      (network.config as any).url,
      args.transactionSpeed,
    );

    publishProofs(
      redis,
      beaconApi,
      contract,
      currentConfig,
      Number(args.slotsJump),
      hashiAdapterContract,
      (network.config as any).url,
      args.transactionSpeed,
    );

    // never resolving promise to block the task
    return new Promise(() => {});
  });
