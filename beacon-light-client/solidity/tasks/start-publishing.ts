import { task } from 'hardhat/config';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { SolidityContract } from '../../../relay/implementations/solidity-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Contract, ethers } from 'ethers';
import hashi_abi from './hashi_abi.json';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';

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
  .addParam(
    'hashi',
    'The address of the Hashi adapter contract',
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
      console.warn('This follownetwork is not specified in networkconfig');
      return;
    }

    const currentConfig = getNetworkConfig(args.follownetwork);

    let publisher;

    if (!args.privatekey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privatekey, ethers.provider);
    }

    console.log('Publishing updates with the account:', publisher.address);
    console.log('Account balance:', (await publisher.getBalance()).toString());

    console.log(`Contract address ${args.lightclient}`);

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
    const beaconApi = new BeaconApi(currentConfig.BEACON_REST_API);
    const contract = new SolidityContract(
      lightClientContract,
      (network.config as any).url,
      args.transactionspeed,
    );

    publishProofs(
      redis,
      beaconApi,
      contract,
      currentConfig[args.follownetwork],
      hashiAdapterContract,
      (network.config as any).url,
      args.transactionspeed,
    );

    // never resolving promise to block the task
    return new Promise(() => {});
  });
