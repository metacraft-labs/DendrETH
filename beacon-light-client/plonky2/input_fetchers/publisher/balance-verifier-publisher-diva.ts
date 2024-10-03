import { Redis } from '@dendreth/relay/implementations/redis';
import { extractHostnameAndPort } from '@dendreth/utils/ts-utils/common-utils';
import JSONbig from 'json-bigint';
import { publishTransaction } from '@dendreth/relay/implementations/publish_evm_transaction';
import Web3 from 'web3';
import http from 'http';
import { RequestOptions } from 'https';
import { ethers } from 'ethers';
import BalanceVerifierDivaAbi from '../abi/balance_verifier_diva_abi.json';
import CONSTANTS from '../../kv_db_constants.json';

import { CommandLineOptionsBuilder } from '../utils/cmdline';
import {
  getBalanceWrapperProofWithPublicInputs,
  getBalanceWrapperVerifierOnly,
} from '../redis_interactions';
(async () => {
  const commandOptions = new CommandLineOptionsBuilder()
    .withRedisOpts()
    .option('rpc-url', {
      describe: 'The RPC URL',
      type: 'string',
      demandOption: true,
    })
    .option('gnark-server-url', {
      describe: 'The URL of the gnark server',
      type: 'string',
      demandOption: true,
    })
    .withProtocolOpts()
    .withTransactionSpeedOpts()
    .option('balance-verifier', {
      describe: 'The address of the balance verifier contract',
      type: 'string',
      demandOption: true,
    })
    .build();

  const rpcUrl: string = commandOptions['rpc-url'];

  const provider = new ethers.providers.JsonRpcProvider(rpcUrl);

  let privateKey = process.env.USER_PRIVATE_KEY;

  if (privateKey === undefined) {
    throw new Error('USER_PRIVATE_KEY unset');
  }

  let publisher = new ethers.Wallet(privateKey, provider);

  let balanceVerifierAddress = commandOptions['balance-verifier']!;

  console.log(`Publishing updates with the account: ${publisher.address}`);
  console.log(`Account balance: ${(await publisher.getBalance()).toString()}`);

  console.log(`Contract address ${balanceVerifierAddress}`);

  const balanceVerifierContract = new ethers.Contract(
    balanceVerifierAddress,
    BalanceVerifierDivaAbi,
    publisher,
  );

  const web3 = new Web3(rpcUrl);

  const redis = new Redis(
    commandOptions['redis-host'],
    commandOptions['redis-port'],
  );

  console.log('Publishing proofs');

  let protocol = commandOptions['protocol'];

  redis.subscribeForGnarkProofs(protocol, async () => {
    console.log('Received new proof');
    let final_layer_proof_string = (await redis.get(
      `${protocol}:${CONSTANTS.depositBalanceVerificationFinalProofKey}`,
    ))!;

    const final_layer_proof_json = JSON.parse(final_layer_proof_string);

    let final_layer_proof_input = JSON.parse(
      (await redis.get(
        `${protocol}:${CONSTANTS.depositBalanceVerificationFinalProofInputKey}`,
      ))!,
    );

    let balance_wrapper_proof_with_public_inputs =
      await getBalanceWrapperProofWithPublicInputs(redis, protocol);
    let balance_wrapper_verifier_only = await getBalanceWrapperVerifierOnly(
      redis,
      protocol,
    );

    const postData = {
      verifier_only_circuit_data: JSONbig.parse(balance_wrapper_verifier_only),
      proof_with_public_inputs: JSONbig.parse(
        balance_wrapper_proof_with_public_inputs,
      ),
    };

    const { hostname, port } = extractHostnameAndPort(
      commandOptions['gnark-server-url'],
    );

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
        let balanceSum = final_layer_proof_json.balanceSum;
        let numberOfNonActivatedValidators =
          final_layer_proof_json.numberOfNonActivatedValidators;
        let numberOfActiveValidators =
          final_layer_proof_json.numberOfActiveValidators;
        let numberOfExitedValidators =
          final_layer_proof_json.numberOfExitedValidators;
        let numberOfSlashedValidators =
          final_layer_proof_json.numberOfSlashedValidators;

        await publishTransaction(
          balanceVerifierContract,
          'verify',
          [
            proof,
            final_layer_proof_input.slot,
            final_layer_proof_input.executionBlockNumber,
            balanceSum,
            numberOfNonActivatedValidators,
            numberOfActiveValidators,
            numberOfExitedValidators,
            numberOfSlashedValidators,
          ],
          web3,
          commandOptions['transaction-speed'],
          true,
        );
      });
    });

    request.write(JSONbig.stringify(postData));
    request.end();
  });
})();
