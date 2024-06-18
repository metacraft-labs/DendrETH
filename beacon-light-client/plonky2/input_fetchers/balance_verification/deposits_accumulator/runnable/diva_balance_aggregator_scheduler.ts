import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import accountManagerAbi from '../../../abi/account_manager_abi.json';
import { ethers } from 'ethers';
import { storeBalanceVerificationData } from '../lib/get_balance_verification_data';

const options = new CommandLineOptionsBuilder()
  .withRedisOpts()
  .withRedisOpts()
  .withBeaconNodeOpts()
  .option('json-rpc', {
    describe: 'The RPC URL',
    type: 'string',
    demandOption: true,
  })
  .option('address', {
    describe: 'The validators accumulator contract address',
    type: 'string',
    default: undefined,
  })
  .withRangeOpts()
  .withProtocolOpts()
  .option('snapshot-contract-address', {
    describe: 'The contract address',
    type: 'string',
    demandOption: true,
  })
  .withBeaconNodeOpts()
  .build();

const snapshotContractAddress = options['snapshot-contract-address'];
const provider = new ethers.providers.JsonRpcProvider(options['rpc-url']);

const snapshot = new ethers.Contract(
  snapshotContractAddress,
  accountManagerAbi,
  provider,
);

snapshot.on('SnapshotTaken', async (_: number, currentSlot: number) => {
  await storeBalanceVerificationData({
    beaconNodeUrls: options['beacon-node'],
    slot: currentSlot,
    take: options['take'],
    offset: options['offset'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
    address: options['address'],
    rpcUrl: options['json-rpc'],
    protocol: options['protocol'],
  });
});
