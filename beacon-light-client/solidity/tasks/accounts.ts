import { task } from 'hardhat/config';
import { getGenericLogger } from '../../../libs/typescript/ts-utils/logger';

const logger = getGenericLogger();

task('accounts', 'Prints the list of accounts', async (_, { ethers }) => {
  logger.info('Getting Signers..');
  (await ethers.getSigners()).map(a => logger.info(a.address));
});
