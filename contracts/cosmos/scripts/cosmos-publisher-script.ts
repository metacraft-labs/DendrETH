import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { Redis } from '@dendreth/relay/implementations/redis';
import { CosmosContract } from '@dendreth/relay/implementations/cosmos-contract';
import { publishProofs } from '@dendreth/relay/on_chain_publisher';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import {
  NetworkConfig,
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';

async function publishTask() {
  const config = {
    REDIS_HOST: process.env.REDIS_HOST || 'localhost',
    REDIS_PORT: Number(process.env.REDIS_PORT) || 6379,
  };

  checkConfig(config);
  const network = process.argv[2];
  const contractAddress = process.argv[3];
  const followNetwork = process.argv[4];
  const slotsJump = Number(process.argv[5]);

  if (!isSupportedFollowNetwork(followNetwork)) {
    console.warn('This follownetwork is not specified in networkconfig');
    return;
  }

  const currentNetwork = await getNetworkConfig(followNetwork as NetworkConfig);

  let address;
  let rpcEndpoint;
  switch (network) {
    case 'cudos': {
      address = String(process.env['CUDOS_PUBLIC_KEY']);
      rpcEndpoint = String(process.env['CUDOS_RPC_ENDPOINT']);
      break;
    }
    case 'malaga': {
      address = String(process.env['MALAGA_ADDRESS']);
      rpcEndpoint = String(process.env['MALAGA_RPC_ENDPOINT']);
      break;
    }
    default: {
      console.error('Incorrect network!');
    }
  }
  console.log('Publishing updates with the account:', address);
  console.log('Account balance:');

  console.log(`Contract address `, contractAddress);

  const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);
  const beaconApi = await getBeaconApi(currentNetwork.BEACON_REST_API!);

  console.log(contractAddress);
  console.log(address);
  console.log(rpcEndpoint);
  console.log(network);

  const contract = new CosmosContract(
    contractAddress,
    address,
    rpcEndpoint,
    network,
  );

  publishProofs(redis, beaconApi, contract, currentNetwork, slotsJump);
}

publishTask();
