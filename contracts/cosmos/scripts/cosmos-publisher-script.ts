import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { CosmosContract } from '../../../relay/implementations/cosmos-contract';
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
  const network = process.argv[2];
  const contractAddress = process.argv[3];
  const followNetwork = process.argv[4];

  const currentNetwork = networkConfig[followNetwork] as Config;

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
  const beaconApi = new BeaconApi(currentNetwork.BEACON_REST_API!);

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

  publishProofs(redis, beaconApi, contract, currentNetwork);
}

publishTask();
