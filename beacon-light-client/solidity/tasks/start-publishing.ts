import { task } from 'hardhat/config';
import { getBeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { SolidityContract } from '../../../relay/implementations/solidity-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Contract, ethers } from 'ethers';
import hashi_abi from './hashi_abi.json';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';
import { getGenericLogger } from '../../../libs/typescript/ts-utils/logger';
import { initPrometheusSetup } from '../../../libs/typescript/ts-utils/prometheus-utils';

const logger = getGenericLogger();

task('start-publishing', 'Run relayer')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .addParam('follownetwork', 'The network the contract follows')
  .addParam(
    'privatekey',
    'The private key that will be used to publish',
    undefined,
    undefined,
    true,
  )
  .addParam(
    'transactionspeed',
    'The speed you want the transactions to be included in a block',
    'avg',
    undefined,
    true,
  )
  .addParam('slotsjump', 'The number of slots to jump')
  .addParam(
    'hashi',
    'The address of the Hashi adapter contract',
    '',
    undefined,
    true,
  )
  .addParam(
    'prometheusport',
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

    if (args.follownetwork !== 'pratter' && args.follownetwork !== 'mainnet') {
      logger.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    if (args.prometheusport) {
      console.log(`Initializing Prometheus on port ${args.prometheusport}`);

      let networkName: string = '';
      for (let i = 0; i < process.argv.length; i++) {
        const arg = process.argv[i];
        if (arg === '--network' && i + 1 < process.argv.length) {
          networkName = process.argv[i + 1];
          break;
        }
      }

      initPrometheusSetup(args.prometheusport, networkName);
    }

    const currentConfig = getNetworkConfig(args.follownetwork);

    let publisher;

    if (!args.privatekey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privatekey, ethers.provider);
    }

    logger.info(`Publishing updates with the account: ${publisher.address}`);
    logger.info(
      `Account balance: ${(await publisher.getBalance()).toString()}`,
    );

    logger.info(`Contract address ${args.lightclient}`);

    const lightClientContract = await ethers.getContractAt(
      'BeaconLightClient',
      args.lightclient,
      publisher,
    );

    if (
      args.transactionspeed &&
      !['slow', 'avg', 'fast'].includes(args.transactionspeed)
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
      args.transactionspeed,
    );

    publishProofs(
      redis,
      beaconApi,
      contract,
      currentConfig,
      Number(args.slotsjump),
      hashiAdapterContract,
      (network.config as any).url,
      args.transactionspeed,
    );

    // never resolving promise to block the task
    return new Promise(() => {});
  });
