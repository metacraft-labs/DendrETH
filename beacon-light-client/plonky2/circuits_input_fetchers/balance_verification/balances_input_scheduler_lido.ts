import { ethers, providers } from 'ethers';
import { getBalancesInput } from './get_balances_input';
import { CommandLineOptionsBuilder } from '../cmdline';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { getLidoWithdrawCredentials } from '@dendreth/utils/balance-verification-utils/utils';
import lidoLocatorAbi from './abi/lido_locator_abi.json';
import accountingOracleAbi from './abi/lido_accounting_oracle_abi.json';
import hashConsensusAbi from './abi/hash_consensus_abi.json';

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

  eventSource.addEventListener('head', async (event: any) => {
    const headSlot = BigInt(JSON.parse(event.data).slot);
    if (headSlot >= nextRefSlot) {
      await getBalancesInput({
        protocol: 'lido',
        withdrawCredentials: LIDO_WITHDRAWAL_CREDENTIALS,
        slot: nextRefSlot,
        beaconNodeUrls: options['beacon-node'],
      });

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

  const [refSlot] = await hashConsensus.getCurrentFrame();
  const epochsPerFrame = await hashConsensus.getFrameConfig()[1];
  const [slotsPerEpoch] = await hashConsensus.getChainConfig();

  return refSlot + epochsPerFrame * slotsPerEpoch;
}
