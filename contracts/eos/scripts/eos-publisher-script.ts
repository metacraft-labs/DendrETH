import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { EOSContract } from '../../../relay/implementations/eos-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import * as networkConfig from '../../../relay/constants/network_config.json';
import { Config } from '../../../relay/constants/constants';

async function publishTask() {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(config);
  const rpcEndpoint = process.argv[2];
  const contractAddress = process.argv[3];
  const followNetwork = process.argv[4];

  const currentNetwork = networkConfig[followNetwork] as Config;

  console.log('Account balance:');

  console.log(`Contract address `, contractAddress);

  const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);
  const beaconApi = new BeaconApi(currentNetwork.BEACON_REST_API!);
  const contract = new EOSContract(contractAddress, rpcEndpoint);
  publishProofs(redis, beaconApi, contract, currentNetwork);
}

publishTask();
