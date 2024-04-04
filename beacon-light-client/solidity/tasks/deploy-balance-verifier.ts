import { task } from 'hardhat/config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  getLidoWithdrawCredentials,
  getGenesisBlockTimestamp,
  isNetwork,
  assertSupportedNetwork,
} from '@dendreth/utils/balance-verification-utils/utils';

const logger = getGenericLogger();

task('deploy-balance-verifier', 'Deploy the beacon light client contract')
  .addParam(
    'withdrawcredentials',
    'The withdraw credentials for the Diva contract',
    undefined,
    undefined,
    true,
  )
  .setAction(async (args, { run, ethers, network }) => {
    await run('compile');
    const [deployer] = await ethers.getSigners();

    logger.info(`Deploying contracts with the account: ${deployer.address}`);
    logger.info(`Account balance: ${(await deployer.getBalance()).toString()}`);

    const networkName = assertSupportedNetwork(network.name);

    let WITHDRAWAL_CREDENTIALS = !args.withdrawcredentials
      ? getLidoWithdrawCredentials(networkName)
      : args.withdrawcredentials;
    let GENESIS_BLOCK_TIMESTAMP = getGenesisBlockTimestamp(networkName);

    const config = {
      REDIS_HOST: process.env.REDIS_HOST,
      REDIS_PORT: Number(process.env.REDIS_PORT),
    };

    checkConfig(config);

    const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

    let balance_wrapper_verifier_only = await redis.get(
      'balance_wrapper_verifier_only',
    );

    if (balance_wrapper_verifier_only === null) {
      logger.error('No wrapper in redis');
      return;
    }

    let VERIFIER_DIGEST = JSON.parse(
      balance_wrapper_verifier_only,
    ).circuit_digest;

    logger.info(
      `Constructor args ${VERIFIER_DIGEST} ${WITHDRAWAL_CREDENTIALS} ${GENESIS_BLOCK_TIMESTAMP}`,
    );

    const beaconLightClient = await (
      await ethers.getContractFactory('BalanceVerifier')
    ).deploy(VERIFIER_DIGEST, WITHDRAWAL_CREDENTIALS, GENESIS_BLOCK_TIMESTAMP);

    logger.info('>>> Waiting for BalanceVerifier deployment...');

    logger.info(
      `Deploying transaction hash.. ${beaconLightClient.deployTransaction.hash}`,
    );

    const contract = await beaconLightClient.deployed();

    logger.info(`>>> ${contract.address}`);
    logger.info('>>> Done!');
  });
