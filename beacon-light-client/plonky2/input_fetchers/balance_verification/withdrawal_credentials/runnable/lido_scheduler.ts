import { BigNumber, ethers, providers } from 'ethers';
import { getBalancesInput } from '../lib/scheduler';
import { CommandLineOptionsBuilder } from '../../../utils/cmdline';
import commonConfig from '../../../../common_config.json';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { getLidoWithdrawCredentials } from '@dendreth/utils/balance-verification-utils/utils';
import lidoLocatorAbi from '../../abi/lido_locator_abi.json';
import accountingOracleAbi from '../../abi/lido_accounting_oracle_abi.json';
import hashConsensusAbi from '../../abi/hash_consensus_abi.json';
import { exec } from 'child_process';
import { promisify } from 'util';

const promisified_exec = promisify(exec);

(async () => {
  const options = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .option('rpc-url', {
      describe: 'The RPC URL',
      type: 'string',
      default: process.env.SEPOLIA_RPC,
    })
    .option('lido-locator-contract-address', {
      describe: 'The contract address',
      type: 'string',
      demandOption: true,
    })
    .withBeaconNodeOpts()
    .option('network', {
      describe: 'The network',
      type: 'string',
      default: 'sepolia',
    })
    .build();

  const provider = new ethers.providers.JsonRpcProvider(options['rpc-url']);
  const lidoLocatorContractAddress = options['lido-locator-contract-address'];

  let nextRefSlot = await getNextRefSlot(provider, lidoLocatorContractAddress);

  const beaconApi = await getBeaconApi(options['beacon-node']);
  const eventSource = beaconApi.subscribeForEvents(['head']);

  const LIDO_WITHDRAWAL_CREDENTIALS = getLidoWithdrawCredentials(
    options['network'],
  );

  let processedSlots = new Set<number>();

  console.log(nextRefSlot);

  eventSource.addEventListener('head', async (event: any) => {
    const headSlot = JSON.parse(event.data).slot;

    if (headSlot >= nextRefSlot && !processedSlots.has(nextRefSlot)) {
      processedSlots.add(nextRefSlot);

      await getBalancesInput({
        protocol: 'lido',
        withdrawCredentials: LIDO_WITHDRAWAL_CREDENTIALS,
        slot: nextRefSlot,
        beaconNodeUrls: options['beacon-node'],
      });

      await promisified_exec('./circuits_executables/run_everywhere.sh lido');

      nextRefSlot = await getNextRefSlot(provider, lidoLocatorContractAddress);
    }
  });
})();

async function getNextRefSlot(
  provider: providers.JsonRpcProvider,
  lidoLocatorContractAddress: string,
): Promise<number> {
  const lidoLocator = new ethers.Contract(
    lidoLocatorContractAddress,
    lidoLocatorAbi,
    provider,
  );

  const accountingOracleAddress = await lidoLocator.accountingOracle();

  const accountingOracle = new ethers.Contract(
    accountingOracleAddress,
    accountingOracleAbi,
    provider,
  );

  const hashConsensusAddress = await accountingOracle.getConsensusContract();

  const hashConsensus = new ethers.Contract(
    hashConsensusAddress,
    hashConsensusAbi,
    provider,
  );

  const refSlot: BigNumber = (await hashConsensus.getCurrentFrame())[0];
  const epochsPerFrame: BigNumber = (await hashConsensus.getFrameConfig())[1];
  const slotsPerEpoch: BigNumber = (await hashConsensus.getChainConfig())[0];

  return refSlot.add(epochsPerFrame.mul(slotsPerEpoch)).toNumber();
}
