import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import accountManagerAbi from '../../../abi/account_manager_abi.json';
import { BigNumber, ethers } from 'ethers';
import { storeBalanceVerificationData } from '../lib/get_balance_verification_data';

const options = new CommandLineOptionsBuilder()
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
const provider = new ethers.providers.JsonRpcProvider(options['json-rpc']);

const snapshot = new ethers.Contract(
  snapshotContractAddress,
  accountManagerAbi,
  provider,
);

console.log('Running diva_balance_aggregator_scheduler:');
console.log('\ttake:', options['take']);
console.log('\toffset:', options['offset']);
console.log('\tredis-host:', options['redis-host']);
console.log('\tredis-port:', options['redis-port']);
console.log('\tredis-auth:', options['redis-auth'].length);
console.log('\taddress:', options['address']);
console.log('\tjson-rpc:', options['json-rpc']);
console.log('\tprotocol:', options['protocol']);
console.log('\tsnapshot-contract-address:', snapshotContractAddress);
console.log();
console.log('Binding to SnapshotTaken events...');

snapshot.on('SnapshotTaken', async (_: BigNumber, currentSlot: BigNumber) => {
  const now: string = (new Date()).toISOString();
  console.log(`${now} | SnapshotTaken received: slot+${currentSlot}`);
  await storeBalanceVerificationData({
    beaconNodeUrls: options['beacon-node'],
    slot: currentSlot.toNumber(),
    take: options['take'],
    offset: options['offset'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
    redisAuth: options['redis-auth'],
    address: options['address'],
    rpcUrl: options['json-rpc'],
    protocol: options['protocol'],
  });
});
