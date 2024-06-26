import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import { getBalancesInput } from '../lib/scheduler';
import accountManagerAbi from '../../abi/account_manager_abi.json';
import validatorManagerAbi from '../../abi/validator_manager_abi.json';
import { ethers } from 'ethers';
import { exec } from 'child_process';
import { promisify } from 'util';
const promisified_exec = promisify(exec);

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
  .withBeaconNodeOpts()
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
  const withdrawalCredentials =
    await validatorManager.getWithdrawalCredentials();

  await getBalancesInput({
    protocol: 'diva',
    withdrawalCredentials,
    slot: currentSlot,
    beaconNodeUrls: options['beacon-node'],
    redisHost: options['redis-host'],
    redisPort: options['redis-port'],
  });

  let run_everywhere_output = await promisified_exec(
    './circuits_executables/run_everywhere.sh diva',
  );

  console.log(run_everywhere_output);
});
