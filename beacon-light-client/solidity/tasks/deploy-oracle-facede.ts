import { task } from 'hardhat/config';

task('deploy', 'Deploy the oracle adapter contract')
  .addParam(
    'lightclientaddress',
    'The address of the light client contract',
    '0x0000000000000000000000000000000000000000',
    undefined,
    true,
  )
  .setAction(async (args, { run, ethers }) => {
    await run('compile');
    const [deployer] = await ethers.getSigners();

    console.log('Deploying contracts with the account:', deployer.address);
    console.log('Account balance:', (await deployer.getBalance()).toString());

    const oracleAdapter = await (
      await ethers.getContractFactory('OracleAdapterFacade')
    ).deploy(args.lightclientaddress);

    console.log('>>> Waiting for OracleAdapterdFacade deployment...');

    console.log(
      'Deploying transaction hash..',
      oracleAdapter.deployTransaction.hash,
    );

    const contract = await oracleAdapter.deployed();

    console.log(`>>> ${contract.address}`);
    console.log('>>> Done!');
  });
