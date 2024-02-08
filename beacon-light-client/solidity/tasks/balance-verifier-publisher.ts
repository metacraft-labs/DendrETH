import { task } from 'hardhat/config';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import { Redis } from '../../../relay/implementations/redis';
import { SolidityContract } from '../../../relay/implementations/solidity-contract';
import { publishProofs } from '../../../relay/on_chain_publisher';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Contract, ethers } from 'ethers';
import hashi_abi from './hashi_abi.json';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';
import { getGenericLogger } from '../../../libs/typescript/ts-utils/logger';
import { initPrometheusSetup } from '../../../libs/typescript/ts-utils/prometheus-utils';
import {
  bitsToBytes,
  hexToBits,
} from '../../../libs/typescript/ts-utils/hex-utils';

const logger = getGenericLogger();

task('balance-verifier-publisher', 'Run relayer')
  .addParam('balanceverifier', 'The address of the BalanceVerifier contract')
  .addParam(
    'privatekey',
    'The private key that will be used to publish',
    undefined,
    undefined,
    true,
  )
  .addParam(
    'transactionspeed',
    'The speed you want the transactions to be included in a block',
    'avg',
    undefined,
    true,
  )
  .addParam(
    'prometheusport',
    'Port No. (3000-3005) for Node Express server where Prometheus is listening.',
    '',
    undefined,
    true,
  )
  .setAction(async (args, { ethers, network }) => {
    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    if (args.prometheusport) {
      console.log(`Initializing Prometheus on port ${args.prometheusport}`);

      let networkName: string = '';
      for (let i = 0; i < process.argv.length; i++) {
        const arg = process.argv[i];
        if (arg === '--network' && i + 1 < process.argv.length) {
          networkName = process.argv[i + 1];
          break;
        }
      }

      initPrometheusSetup(args.prometheusport, networkName);
    }

    let publisher;

    if (!args.privatekey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privatekey, ethers.provider);
    }

    logger.info(`Publishing updates with the account: ${publisher.address}`);
    logger.info(
      `Account balance: ${(await publisher.getBalance()).toString()}`,
    );

    logger.info(`Contract address ${args.balanceverifier}`);

    const balanceVerifierContract = await ethers.getContractAt(
      'BalanceVerifier',
      args.balanceverifier,
      publisher,
    );

    if (
      args.transactionspeed &&
      !['slow', 'avg', 'fast'].includes(args.transactionspeed)
    ) {
      throw new Error('Invalid transaction speed');
    }

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    const contract = new SolidityContract(
      balanceVerifierContract,
      (network.config as any).url,
      args.transactionspeed,
    );

    // TODO: publish proofs
    console.log('Publishing proofs');

    let final_layer_proof = JSON.parse((await redis.get('final_layer_proof'))!);

    console.log(final_layer_proof);

    let stateRoot = bitsToBytes(final_layer_proof.stateRoot);

    let balance_sum =
      BigInt(final_layer_proof.balanceSum[0]) |
      (BigInt(final_layer_proof.balanceSum[1]) << 32n);

    let numberOfNonActivatedValidators =
      final_layer_proof.numberOfNonActivatedValidators;
    let numberOfActiveValidators = final_layer_proof.numberOfActiveValidators;
    let numberOfExitedValidators = final_layer_proof.numberOfExitedValidators;

    console.log('State root', stateRoot);
    console.log('Balance sum', balance_sum);
    console.log(
      'Number of non activated validators',
      numberOfNonActivatedValidators,
    );
    console.log('Number of active validators', numberOfActiveValidators);
    console.log('Number of exited validators', numberOfExitedValidators);

    // never resolving promise to block the task
    return new Promise(() => {});
  });
