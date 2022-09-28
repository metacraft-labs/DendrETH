import { task } from 'hardhat/config';

task('accounts', 'Prints the list of accounts', async (_, { ethers }) => {
  (await ethers.getSigners()).map(a => console.log(a.address));
});
