import { task } from 'hardhat/config';
import { getGenericLogger } from '@dendreth/utils/ts-utils/logger';
import { checkConfig } from '@dendreth/utils/ts-utils/common-utils';
import { Redis } from '@dendreth/relay/implementations/redis';
import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import { ethers } from 'ethers';

const logger = getGenericLogger();

task('deploy-balance-verifier', 'Deploy the beacon light client contract')
  .addParam('protocol', 'The protocol used. Should be "lido" or "diva"')
  .addParam(
    'ownerAddress',
    'The address of the owner of the balance verifier contract',
  )
  .addParam('beaconNode', 'The endpoint of the beacon node API for the network')
  .addParam('rpcUrl', 'The RPC URL for the network')
  .addParam(
    'additionalData',
    'Either the withdraw credentials for the Lido contract or the accumulator contract address for the Diva contract',
  )
  .addOptionalParam(
    'verifierDigest',
    'The verifier digest for the plonky2 circuit to initialize the BalanceVerifier contract',
  )
  .setAction(async (args, { run, ethers }) => {
    await run('compile');
    const provider = new ethers.providers.JsonRpcProvider(args.rpcUrl);
    const deployer = new ethers.Wallet(process.env.USER_PRIVATE_KEY!, provider);

    logger.info(`Deploying contracts with the account: ${deployer.address}`);
    logger.info(`Account balance: ${(await deployer.getBalance()).toString()}`);

    const beaconApi = await getBeaconApi([args.beaconNode]);

    const genesisTime = (await beaconApi.getGenesisData()).genesisTime;

    let VERIFIER_DIGEST = args.verifierDigest;
    if (!VERIFIER_DIGEST) {
      const config = {
        REDIS_HOST: process.env.REDIS_HOST,
        REDIS_PORT: Number(process.env.REDIS_PORT),
      };

      checkConfig(config);

      const redis = new Redis(config.REDIS_HOST!, config.REDIS_PORT);

      let balance_wrapper_verifier_only = await redis.get(
        `${args.protocol}:balance_wrapper_verifier_only`,
      );

      if (balance_wrapper_verifier_only === null) {
        logger.error('No wrapper in redis');
        return;
      }

      VERIFIER_DIGEST = JSON.parse(
        balance_wrapper_verifier_only,
      ).circuit_digest;
    }

    const protocol = args.protocol;
    if (protocol !== 'lido' && protocol !== 'diva') {
      logger.error('Invalid protocol');
      return;
    }

    const verifier = await (await ethers.getContractFactory('PlonkVerifier'))
      .connect(deployer)
      .deploy();

    let beaconLightClient: ethers.Contract;
    let contractName: string;
    if (protocol === 'diva') {
      if (!ethers.utils.isAddress(args.additionalData)) {
        logger.error('Invalid accumulator address');
        return;
      }

      const accumulatorAddress = args.additionalData;
      logger.info(
        `Constructor args ${VERIFIER_DIGEST} ${genesisTime} ${verifier.address} ${accumulatorAddress} ${args.ownerAddress}`,
      );

      contractName = 'BalanceVerifierDiva';
      beaconLightClient = await (await ethers.getContractFactory(contractName))
        .connect(deployer)
        .deploy(
          VERIFIER_DIGEST,
          genesisTime,
          verifier.address,
          accumulatorAddress,
          args.ownerAddress,
        );
    } else {
      if (
        args.additionalData.length !== 66 ||
        !ethers.utils.isHexString(args.additionalData)
      ) {
        logger.error('Invalid withdrawal credentials');
        return;
      }

      const withdrawCredentials = args.additionalData;
      logger.info(
        `Constructor args ${VERIFIER_DIGEST} ${withdrawCredentials} ${genesisTime} ${verifier.address} ${args.ownerAddress}`,
      );
      contractName = 'BalanceVerifierLido';

      beaconLightClient = await (await ethers.getContractFactory(contractName))
        .connect(deployer)
        .deploy(
          VERIFIER_DIGEST,
          withdrawCredentials,
          genesisTime,
          verifier.address,
          args.ownerAddress,
        );
    }

    logger.info(`>>> Waiting for ${contractName} deployment...`);

    logger.info(
      `Deploying transaction hash.. ${beaconLightClient.deployTransaction.hash}`,
    );

    const contract = await beaconLightClient.deployed();

    logger.info(`>>> ${contract.address}`);
    logger.info('>>> Done!');
  });
