import { task } from "hardhat/config";
import { getConstructorArgs } from "./utils";

task("deploy", "Deploy the beacon light client contract")
    .setAction(async (_, { run, ethers, network }) => {
        await run('compile');
        const [deployer] = await ethers.getSigners();

        const beaconLightClient = await (await ethers.getContractFactory('BeaconLightClient'))
            .deploy(...getConstructorArgs(network.name));

        console.log('>>> Waiting for BeaconLightClient deployment...');

        await beaconLightClient.deployed();

        console.log(`>>> ${beaconLightClient.address}`);
        console.log('>>> Done!');
    });
