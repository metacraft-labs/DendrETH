import { task } from 'hardhat/config';
import {
  BeaconApi,
  getBeaconApi,
} from '@dendreth/relay/implementations/beacon-api';
import { getSlotOnChain } from '@dendreth/relay/utils/smart_contract_utils';
import { SolidityContract } from '@dendreth/relay/implementations/solidity-contract';
import { Contract, ethers } from 'ethers';
import hashi_abi from './hashi_abi.json';
import {
  getNetworkConfig,
  isSupportedFollowNetwork,
} from '@dendreth/relay/utils/get_current_network_config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import {
  findClosestValidBlock,
  getInputFromTo,
} from '@dendreth/relay/workers/poll-updates/get_light_client_input_from_to';
import {
  accountBalanceGauge,
  initPrometheusSetup,
  registerGaugesForStartPublishing,
  startResourceMetricsUpdate,
} from '@dendreth/utils/ts-utils/prometheus-utils';
import { computeSyncCommitteePeriodAt } from '@dendreth/utils/ts-utils/ssz-utils';
import { readFile, writeFile } from 'fs/promises';
import { exec as _exec } from 'child_process';
import { promisify } from 'util';

const exec = promisify(_exec);

const logger = getGenericLogger();

task('update-sync-committe', 'Run relayer')
  .addParam('lightClient', 'The address of the BeaconLightClient contract')
  .addParam('followNetwork', 'The network the contract follows')
  .addParam(
    'privateKey',
    'The private key that will be used to publish',
    undefined,
    undefined,
    true,
  )
  .addParam(
    'transactionSpeed',
    'The speed you want the transactions to be included in a block',
    'avg',
    undefined,
    true,
  )
  .addParam(
    'hashi',
    'The address of the Hashi adapter contract',
    '',
    undefined,
    true,
  )
  .addParam(
    'prometheusPort',
    'Port No. (3000-3005) for Node Express server where Prometheus is listening.',
    '',
    undefined,
    true,
  )
  .setAction(async (args, { ethers, network }) => {
    if (!isSupportedFollowNetwork(args.followNetwork)) {
      logger.warn('This followNetwork is not specified in networkconfig');
      return;
    }

    let networkName: string = '';

    if (args.prometheusPort) {
      console.log(`Initializing Prometheus on port ${args.prometheusPort}`);

      for (let i = 0; i < process.argv.length; i++) {
        const arg = process.argv[i];
        if (arg === '--follow-network' && i + 1 < process.argv.length) {
          networkName = process.argv[i + 1];
          break;
        }
      }

      initPrometheusSetup(args.prometheusPort, networkName);
      registerGaugesForStartPublishing();
      startResourceMetricsUpdate(networkName);
    }

    const currentConfig = await getNetworkConfig(args.followNetwork);

    let publisher;

    if (!args.privateKey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privateKey, ethers.provider);
    }

    logger.info(`Publishing updates with the account: ${publisher.address}`);
    const accountBalance = await publisher.getBalance();

    logger.info(`Account balance: ${accountBalance.toString()}`);
    accountBalanceGauge.labels(networkName).set(Number(accountBalance));

    logger.info(`Contract address ${args.lightClient}`);

    const lightClientContract = await ethers.getContractAt(
      'BeaconLightClient',
      args.lightClient,
      publisher,
    );

    if (
      args.transactionSpeed &&
      !['slow', 'avg', 'fast'].includes(args.transactionSpeed)
    ) {
      throw new Error('Invalid transaction speed');
    }

    let hashiAdapterContract: ethers.Contract | undefined;

    if (args.hashi) {
      hashiAdapterContract = new Contract(args.hashi, hashi_abi, publisher);
    }

    const beaconApi = await getBeaconApi(currentConfig.BEACON_REST_API);
    const contract = new SolidityContract(
      lightClientContract,
      (network.config as any).url,
      args.transactionSpeed,
    );

    while (true) {
      logger.info('Getting OnChain Slot..');
      const optimisticSlot = await getSlotOnChain(contract, beaconApi);

      logger.info('Getting CurrentHeadSlot');
      const headSlot = await beaconApi.getCurrentHeadSlot();
      const finalizedSlot = await beaconApi.getCurrentFinalizedSlot();

      const periodAtFinalizedSlot = computeSyncCommitteePeriodAt(
        BigInt(finalizedSlot),
        BigInt(currentConfig.SLOTS_PER_SYNC_COMMITTEE_PERIOD),
      );

      const periodAtOptimisticSlot = computeSyncCommitteePeriodAt(
        BigInt(optimisticSlot),
        BigInt(currentConfig.SLOTS_PER_SYNC_COMMITTEE_PERIOD),
      );

      logger.info(`period at finalized slot ${periodAtFinalizedSlot}`);
      logger.info(`period at optimistic slot ${periodAtOptimisticSlot}`);
      logger.info(`finalized slot ${finalizedSlot}`);
      logger.info(`optimistic slot ${optimisticSlot}`);

      if (periodAtFinalizedSlot == periodAtOptimisticSlot) {
        break;
      }

      const finalizedSlotAtContract = (
        await beaconApi.getFinalizedBlockHeader(optimisticSlot)
      ).slot;

      const nextSlot = await getNextSlot(
        finalizedSlotAtContract,
        finalizedSlot,
        headSlot,
        beaconApi,
      );

      logger.info('Next Slot:', nextSlot);

      if (optimisticSlot >= nextSlot) {
        logger.info('No new enough slots');
        process.exit(0);
      }
      logger.info('Getting proof input..');

      const result = await getInputFromTo(
        optimisticSlot,
        nextSlot,
        beaconApi,
        currentConfig,
      );

      logger.info('Got proof input');

      logger.info('Generating witness');

      // write the input to a file
      await writeFile('input.json', JSON.stringify(result.proofInput), 'utf8');

      await exec('cp $GIT_ROOT/data/light_client.dat ./');

      const witness_result = await exec('light_client input.json witness.wtns');

      logger.info('Witness generated');

      if (witness_result.stdout) {
        logger.info('Witness generation output:', witness_result.stdout);
      }
      if (witness_result.stderr) {
        logger.info('Witness generation error:', witness_result.stderr);
      }

      logger.info('Generating proof');

      const prover_result = await exec(
        'prover $GIT_ROOT/data/light_client.zkey witness.wtns proof.json public.json',
      );

      if (prover_result.stdout) {
        logger.info('Prover output:', prover_result.stdout);
      }

      if (prover_result.stderr) {
        logger.info('Prover error:', prover_result.stderr);
      }

      logger.info('Proof generated');

      const proof = JSON.parse(
        await readFile('proof.json', { encoding: 'utf8' }),
      );

      const update = {
        attestedHeaderRoot:
          '0x' +
          BigInt('0b' + result.proofInput.nextHeaderHash.join(''))
            .toString(16)
            .padStart(64, '0'),
        attestedHeaderSlot: result.proofInput.nextHeaderSlot,
        finalizedHeaderRoot:
          '0x' +
          BigInt('0b' + result.proofInput.finalizedHeaderRoot.join(''))
            .toString(16)
            .padStart(64, '0'),
        finalizedExecutionStateRoot:
          '0x' +
          BigInt('0b' + result.proofInput.execution_state_root.join(''))
            .toString(16)
            .padStart(64, '0'),
      };

      await contract.postUpdateOnChain({
        ...update,
        a: proof.pi_a,
        b: proof.pi_b,
        c: proof.pi_c,
      });

      const transactionSlot = result.proofInput.nextHeaderSlot;

      const currentHeadSlot = await beaconApi.getCurrentHeadSlot();

      logger.info(`Previous slot on the chain ${optimisticSlot}`);

      logger.info(`Transaction publishing for slot ${transactionSlot}`);

      logger.info(`Current slot on the network is ${currentHeadSlot}`);

      let prevSlotBehind =
        ((currentHeadSlot - optimisticSlot) * currentConfig.SECONDS_PER_SLOT) /
        60;
      logger.info(`Prev slot is ${prevSlotBehind} minutes behind`);

      let transactionBehind =
        ((currentHeadSlot - transactionSlot) * currentConfig.SECONDS_PER_SLOT) /
        60;
      logger.info(`Transaction is ${transactionBehind} minutes behind`);
    }
  });

async function getNextSlot(
  finalizedSlotAtContract: number,
  finalizedSlot: number,
  headSlot: number,
  beaconApi: BeaconApi,
) {
  const slotsPerPeriod = await beaconApi.getSlotsPerSyncCommitteePeriod();
  const slotsPerEpoch = await beaconApi.getSlotsPerEpoch();
  const periodAtSlot = computeSyncCommitteePeriodAt(
    BigInt(finalizedSlotAtContract),
    slotsPerPeriod,
  );
  const periodAtFinalizedSlot = computeSyncCommitteePeriodAt(
    BigInt(finalizedSlot),
    slotsPerPeriod,
  );

  console.log('period at slot', periodAtSlot);
  console.log('period at finalized slot', periodAtFinalizedSlot);

  if (periodAtSlot == periodAtFinalizedSlot) {
    logger.info('Finalized slot is in the same period as the contract slot');
    process.exit(0);
  }

  if (periodAtSlot + 1n == periodAtFinalizedSlot) {
    // next slot will be the finalizedSlot
    const potentialNewSlot = finalizedSlot;

    const result = await findClosestValidBlock(
      potentialNewSlot,
      beaconApi,
      headSlot,
    );

    return result.nextBlockHeader.slot;
  }

  // next slot will be the first slot of the last epoch of the next period
  const potentialNewSlot =
    BigInt(periodAtSlot + 1n) * slotsPerPeriod +
    (slotsPerPeriod - slotsPerEpoch);

  console.log('potential new slot', potentialNewSlot);

  console.log('finalized slot', finalizedSlot);

  const result = await findClosestValidBlock(
    Number(potentialNewSlot),
    beaconApi,
    headSlot,
  );

  return result.nextBlockHeader.slot;
}
