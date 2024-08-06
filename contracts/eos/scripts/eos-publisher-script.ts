import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { EOSContract } from '@dendreth/relay/implementations/eos-contract';
import { publishProofs } from '@dendreth/relay/on_chain_publisher';
import {
  NetworkConfig,
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';

async function publishTask() {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
    SLOT_JUMP: Number(process.env.SLOT_JUMP),
  };

  checkConfig(config);
  const rpcEndpoint = process.argv[2];
  const contractAddress = process.argv[3];
  const followNetwork = process.argv[4];

  if (!isSupportedFollowNetwork(followNetwork)) {
    console.warn('This follownetwork is not specified in networkconfig');
    return;
  }

  const currentNetwork = await getNetworkConfig(followNetwork as NetworkConfig);

  console.log('Account balance:');

  console.log(`Contract address `, contractAddress);

  const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);
  const beaconApi = await getBeaconApi(currentNetwork.BEACON_REST_API!);
  const contract = new EOSContract(contractAddress, rpcEndpoint);
  publishProofs(redis, beaconApi, contract, currentNetwork, config.SLOT_JUMP);
}

publishTask();
