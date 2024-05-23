import { task } from 'hardhat/config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Redis } from '@dendreth/relay/implementations/redis';
import {
  getLidoWithdrawCredentials,
  getGenesisBlockTimestamp,
  assertSupportedNetwork,
} from '@dendreth/utils/balance-verification-utils/utils';

const logger = getGenericLogger();

task('deploy-balance-verifier', 'Deploy the beacon light client contract')
  .addParam('protocol', 'The protocol used. Should be "lido" or "diva"')
  .addParam(
    'ownerAddress',
    'The address of the owner of the balance verifier contract',
  )
  .addOptionalParam(
    'withdrawCredentials',
    'The withdraw credentials for the Diva contract',
  )
  .addOptionalParam(
    'verifierDigest',
    'The verifier digest for the plonky2 circuit to initialize the BalanceVerifier contract',
  )
  .setAction(async (args, { run, ethers, network }) => {
    await run('compile');
    const [deployer] = await ethers.getSigners();

    logger.info(`Deploying contracts with the account: ${deployer.address}`);
    logger.info(
      `Account balance: ${(
        await deployer.provider.getBalance(deployer.address)
      ).toString()}`,
    );

    const networkName = assertSupportedNetwork(network.name);

    let WITHDRAWAL_CREDENTIALS = !args.withdrawCredentials
      ? getLidoWithdrawCredentials(networkName)
      : args.withdrawCredentials;
    let GENESIS_BLOCK_TIMESTAMP = getGenesisBlockTimestamp(networkName);

    let VERIFIER_DIGEST = args.verifierDigest;
    if (!VERIFIER_DIGEST) {
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

      VERIFIER_DIGEST = JSON.parse(
        balance_wrapper_verifier_only,
      ).circuit_digest;
    }

    let protocol = args.protocol;
    let CONTRACT = 'BalanceVerifierLido';
    if (protocol !== 'lido' && protocol !== 'diva') {
      logger.error('Invalid protocol');
      return;
    }
    if (protocol === 'diva') {
      CONTRACT = 'BalanceVerifierDiva';
    }

    const verifier = await (
      await ethers.getContractFactory('PlonkVerifier')
    ).deploy();

    logger.info(
      `Constructor args ${VERIFIER_DIGEST} ${WITHDRAWAL_CREDENTIALS} ${GENESIS_BLOCK_TIMESTAMP} ${verifier.address} ${args.ownerAddress}`,
    );

    const beaconLightClient = await (
      await ethers.getContractFactory(CONTRACT)
    ).deploy(
      VERIFIER_DIGEST,
      WITHDRAWAL_CREDENTIALS,
      GENESIS_BLOCK_TIMESTAMP,
      verifier.target,
      args.ownerAddress,
    );

    logger.info(`>>> Waiting for ${CONTRACT} deployment...`);

    logger.info(
      `Deploying transaction hash.. ${
        beaconLightClient.deploymentTransaction()?.hash
      }`,
    );

    const contract = await beaconLightClient.waitForDeployment();

    logger.info(`>>> ${contract.target}`);
    logger.info('>>> Done!');
  });
