import { task } from 'hardhat/config';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { SolidityContract } from '../../../relay/implementations/solidity-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';

task('start-publishing', 'Run relayer')
  .addParam('lightclient', 'The address of the BeaconLightClient contract')
  .addParam('beaconapi', 'The beacon api the contract follows')
  .setAction(async (args, { ethers }) => {
    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    const [publisher] = await ethers.getSigners();

    console.log('Publishing updates with the account:', publisher.address);
    console.log('Account balance:', (await publisher.getBalance()).toString());

    console.log(`Contract address ${args.lightclient}`);

    const lightClientContract = await ethers.getContractAt(
      'BeaconLightClient',
      args.lightclient,
      publisher,
    );

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);
    const beaconApi = new BeaconApi(args.beaconapi);
    const contract = new SolidityContract(lightClientContract);

    publishProofs(redis, beaconApi, contract);

    // never resolving promise to block the task lets see if it works
    return new Promise(() => {});
  });
