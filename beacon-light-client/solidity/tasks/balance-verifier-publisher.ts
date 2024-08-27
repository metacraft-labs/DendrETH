import { task } from 'hardhat/config';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  checkConfig,
  extractHostnameAndPort,
  getBigIntFromLimbs,
} from '@dendreth/utils/ts-utils/common-utils';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { initPrometheusSetup } from '@dendreth/utils/ts-utils/prometheus-utils';
import JSONbig from 'json-bigint';
import { publishTransaction } from '@dendreth/relay/implementations/publish_evm_transaction';
import Web3 from 'web3';
import assert from 'assert';
import http from 'http';
import { RequestOptions } from 'https';

const logger = getGenericLogger();

task('balance-verifier-publisher', 'Run relayer')
  .addParam('balanceVerifier', 'The address of the BalanceVerifier contract')
  .addParam(
    'privateKey',
    'The private key that will be used to publish',
    undefined,
    undefined,
    true,
  )
  .addParam(
    'gnarkServerUrl',
    'The url of the gnark server',
    'http://localhost:3333',
    undefined,
    true,
  )
  .addParam('protocol', 'The protocol used')
  .addParam(
    'transactionSpeed',
    'The speed you want the transactions to be included in a block',
    'avg',
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
    const config = {
      REDIS_HOST: process.env.REDIS_HOST || 'localhost',
      REDIS_PORT: Number(process.env.REDIS_PORT) || 6379,
    };

    checkConfig(config);

    if (args.prometheusPort) {
      console.log(`Initializing Prometheus on port ${args.prometheusPort}`);

      let networkName: string = '';
      for (let i = 0; i < process.argv.length; i++) {
        const arg = process.argv[i];
        if (arg === '--network' && i + 1 < process.argv.length) {
          networkName = process.argv[i + 1];
          break;
        }
      }

      initPrometheusSetup(args.prometheusPort, networkName);
    }

    let publisher;

    if (!args.privateKey) {
      [publisher] = await ethers.getSigners();
    } else {
      publisher = new ethers.Wallet(args.privateKey, ethers.provider);
    }

    logger.info(`Publishing updates with the account: ${publisher.address}`);
    logger.info(
      `Account balance: ${(await publisher.getBalance()).toString()}`,
    );

    logger.info(`Contract address ${args.balanceVerifier}`);

    const balanceVerifierContract = await ethers.getContractAt(
      'contracts/balance_verifier/BalanceVerifierDiva.sol:BalanceVerifierDiva',
      args.balanceVerifier,
      publisher,
    );

    const web3 = new Web3((network.config as any).url);

    console.log('url', (network.config as any).url);

    if (
      args.transactionSpeed &&
      !['slow', 'avg', 'fast'].includes(args.transactionSpeed)
    ) {
      throw new Error('Invalid transaction speed');
    }

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    console.log('Publishing proofs');

    let protocol = args.protocol;

    redis.subscribeForGnarkProofs(protocol, async () => {
      console.log('Received new proof');
      let final_layer_proof = JSON.parse(
        (await redis.get(`${protocol}:final_layer_proof`))!,
      );

      let final_layer_proof_input = JSON.parse(
        (await redis.get(`${protocol}:final_proof_input`))!,
      );

      let balance_wrapper_proof_with_public_inputs =
        await redis.getBalanceWrapperProofWithPublicInputs(protocol);
      let balance_wrapper_verifier_only =
        await redis.getBalanceWrapperVerifierOnly();

      const postData = {
        verifier_only_circuit_data: JSONbig.parse(
          balance_wrapper_verifier_only,
        ),
        proof_with_public_inputs: JSONbig.parse(
          balance_wrapper_proof_with_public_inputs,
        ),
      };

      const { hostname, port } = extractHostnameAndPort(args.gnarkServerUrl);

      const options: RequestOptions = {
        hostname: hostname,
        port: port,
        path: '/genProof',
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      };

      let proof: number[] = [];

      let request = http.request(options, res => {
        console.log('response received');
        res.on('data', chunk => {
          proof.push(...chunk);
        });

        res.on('end', async () => {
          let balanceSum = final_layer_proof.balanceSum;
          let numberOfNonActivatedValidators =
            final_layer_proof.numberOfNonActivatedValidators;
          let numberOfActiveValidators =
            final_layer_proof.numberOfActiveValidators;
          let numberOfExitedValidators =
            final_layer_proof.numberOfExitedValidators;
          let numberOfSlashedValidators =
            final_layer_proof.numberOfSlashedValidators;

          await publishTransaction(
            balanceVerifierContract,
            'verify',
            [
              proof,
              final_layer_proof_input.slot,
              balanceSum,
              numberOfNonActivatedValidators,
              numberOfActiveValidators,
              numberOfExitedValidators,
              numberOfSlashedValidators,
            ],
            web3,
            args.transactionSpeed,
            true,
          );
        });
      });

      request.write(JSONbig.stringify(postData));
      request.end();
    });

    // never resolving promise to block the task
    return new Promise(() => {});
  });
