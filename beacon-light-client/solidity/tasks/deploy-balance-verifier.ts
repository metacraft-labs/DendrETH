import { task } from 'hardhat/config';
import { getNetworkConfig } from '../../../relay/utils/get_current_network_config';
import { getGenericLogger } from '../../../libs/typescript/ts-utils/logger';

const logger = getGenericLogger();

task(
  'deploy-balance-verifier',
  'Deploy the beacon light client contract',
).setAction(async (_, { run, ethers }) => {
  await run('compile');
  const [deployer] = await ethers.getSigners();

  logger.info(`Deploying contracts with the account: ${deployer.address}`);
  logger.info(`Account balance: ${(await deployer.getBalance()).toString()}`);

  const beaconLightClient = await (
    await ethers.getContractFactory('BalanceVerifier')
  ).deploy(
    '12132998113779983235430917548537520464854579851393401583800381700464695543790',
    '0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b',
  );

  logger.info('>>> Waiting for BalanceVerifier deployment...');

  logger.info(
    `Deploying transaction hash.. ${beaconLightClient.deployTransaction.hash}`,
  );

  const contract = await beaconLightClient.deployed();

  logger.info(`>>> ${contract.address}`);
  logger.info('>>> Done!');
});
