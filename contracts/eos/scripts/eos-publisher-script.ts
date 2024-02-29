import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { EOSContract } from '../../../relay/implementations/eos-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';

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

  if (followNetwork !== 'mainnet' && followNetwork !== 'pratter') {
    console.warn('This follownetwork is not specified in networkconfig');
    return;
  }

  const currentNetwork = getNetworkConfig(followNetwork);

  console.log('Account balance:');

  console.log(`Contract address `, contractAddress);

  const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);
  const beaconApi = new BeaconApi(currentNetwork.BEACON_REST_API!);
  const contract = new EOSContract(contractAddress, rpcEndpoint);
  publishProofs(redis, beaconApi, contract, currentNetwork, config.SLOT_JUMP);
}

publishTask();
