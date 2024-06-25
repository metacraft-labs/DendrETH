import { task } from 'hardhat/config';

task('run-snapshot-emitter', 'Either deploy the contract or emit an event')
  .addOptionalParam(
    'address',
    'The address of the contract (will deploy the contract if missing)',
  )
  .addOptionalParam('slot', 'The slot to emit the event with')
  .addOptionalParam('era', 'The era to emit the event with')
  .setAction(async (args, { run, ethers }) => {
    if (!args.address) {
      await run('compile');

      const [deployer] = await ethers.getSigners();
      const snapshotEmitter = await ethers.deployContract('SnapshotEmitter', {
        signer: deployer,
      });
      await snapshotEmitter.deployed();
      console.log(`SnapshotEmitter deployed to ${snapshotEmitter.address}`);
    } else {
      const [deployer] = await ethers.getSigners();
      const snapshotEmitter = await ethers.getContractAt(
        'SnapshotEmitter',
        args.address,
        deployer,
      );

      await snapshotEmitter.emitSnapshot(args.era ?? 0, args.slot ?? 0);
    }
  });
