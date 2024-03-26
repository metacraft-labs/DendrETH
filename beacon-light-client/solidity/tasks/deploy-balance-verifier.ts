import { task } from 'hardhat/config';
import { getGenericLogger } from '../../../libs/typescript/ts-utils/logger';
import { checkConfig } from '../../../libs/typescript/ts-utils/common-utils';
import { Redis } from '../../../relay/implementations/redis';

const logger = getGenericLogger();

task(
  'deploy-balance-verifier',
  'Deploy the beacon light client contract',
).setAction(async (args, { run, ethers, network }) => {
  await run('compile');
  const [deployer] = await ethers.getSigners();

  logger.info(`Deploying contracts with the account: ${deployer.address}`);
  logger.info(`Account balance: ${(await deployer.getBalance()).toString()}`);

  let LIDO_WITHDRAWAL_CREDENTIALS;
  let GENESIS_BLOCK_TIMESTAMP;

  if (network.name === 'mainnet') {
    LIDO_WITHDRAWAL_CREDENTIALS =
      '0x010000000000000000000000b9d7934878b5fb9610b3fe8a5e441e8fad7e293f';
    GENESIS_BLOCK_TIMESTAMP = '1606824023';
  } else if (network.name === 'goerli') {
    LIDO_WITHDRAWAL_CREDENTIALS =
      '0x010000000000000000000000dc62f9e8C34be08501Cdef4EBDE0a280f576D762';
    GENESIS_BLOCK_TIMESTAMP = '1616508000';
  } else if (network.name === 'sepolia') {
    LIDO_WITHDRAWAL_CREDENTIALS =
      '0x010000000000000000000000De7318Afa67eaD6d6bbC8224dfCe5ed6e4b86d76';
    GENESIS_BLOCK_TIMESTAMP = '1655733600';
  } else {
    logger.error('Unsupported network');
    return;
  }

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

  logger.log(
    'Constructor args',
    VERIFIER_DIGEST,
    LIDO_WITHDRAWAL_CREDENTIALS,
    GENESIS_BLOCK_TIMESTAMP,
  );

  const beaconLightClient = await (
    await ethers.getContractFactory('BalanceVerifier')
  ).deploy(
    VERIFIER_DIGEST,
    LIDO_WITHDRAWAL_CREDENTIALS,
    GENESIS_BLOCK_TIMESTAMP,
  );

  logger.info('>>> Waiting for BalanceVerifier deployment...');

  logger.info(
    `Deploying transaction hash.. ${beaconLightClient.deployTransaction.hash}`,
  );

  const contract = await beaconLightClient.deployed();

  logger.info(`>>> ${contract.address}`);
  logger.info('>>> Done!');
});
