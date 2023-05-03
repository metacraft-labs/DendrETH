import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { CosmosContract } from '../../../relay/implementations/cosmos-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';

async function publishTask() {
  const config = {
    BEACON_REST_API: process.env.BEACON_REST_API,
    REDIS_HOST: process.env.REDIS_HOST,
    REDIS_PORT: Number(process.env.REDIS_PORT),
  };

  checkConfig(config);
  const network = process.argv[3];
  const contractAddress = process.argv[4];
  let address;
  let rpcEndpoint;
  let prefix;
  switch (network) {
    case 'cudos': {
      address = String(process.env['CUDOS_PUBLIC_KEY']);
      rpcEndpoint = String(process.env['CUDOS_RPC_ENDPOINT']);
      prefix = 'cudos';
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
  const beaconApi = new BeaconApi(config.BEACON_REST_API!);
  const contract = new CosmosContract(
    contractAddress,
    address,
    rpcEndpoint,
    prefix,
  );
  publishProofs(redis, beaconApi, contract);
}

publishTask();
