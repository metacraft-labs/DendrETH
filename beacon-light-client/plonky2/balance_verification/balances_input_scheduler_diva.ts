import { ethers } from 'ethers';
import { CommandLineOptionsBuilder } from '../cmdline';
import config from '../common_config.json';
import { getBalancesInput } from './get_balances_input';
import accountManagerAbi from './abi/account_manager_abi.json';
import validatorManagerAbi from './abi/validator_manager_abi.json';

const options = new CommandLineOptionsBuilder()
  .withRedisOpts()
  .option('rpc-url', {
    describe: 'The RPC URL',
    type: 'string',
    default: process.env.SEPOLIA_RPC,
  })
  .option('snapshot-contract-address', {
    describe: 'The contract address',
    type: 'string',
    demandOption: true,
  })
  .option('validator-manager-contract-address', {
    describe: 'The validator manager contract address',
    type: 'string',
    demandOption: true,
  })
  .option('beacon-node', {
    alias: 'beacon-node',
    describe: 'The beacon node url',
    type: 'array',
    default: config['beacon-node'],
    description: 'Sets a custom beacon node url',
  })
  .build();

const snapshotContractAddress = options['snapshot-contract-address'];
const provider = new ethers.providers.JsonRpcProvider(options['rpc-url']);

const snapshot = new ethers.Contract(
  snapshotContractAddress,
  accountManagerAbi,
  provider,
);

const validatorManager = new ethers.Contract(
  options['validator-manager-contract-address'],
  validatorManagerAbi,
  provider,
);

snapshot.on('SnapshotTaken', async (_: number, currentSlot: number) => {
  const withdrawCredentials = await validatorManager.getWithdrawalCredentials();

  await getBalancesInput({
    withdrawCredentials,
    slot: currentSlot,
    beaconNodeUrls: options['beacon-node'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
  });
});
