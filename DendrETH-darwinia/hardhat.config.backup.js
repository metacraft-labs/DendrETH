require("@nomiclabs/hardhat-waffle");
const { task } = require("hardhat/config");
const fs = require('fs');


task("accounts", "Prints the list of accounts", async (taskArgs, hre) => {
  const accounts = await hre.ethers.getSigners();

  for (const account of accounts) {
    console.log(account.address);
  }
});


task("deploy", "Deploy contracts on a provided network").setAction(
  async (taskArguments, hre, runSuper) => {
    await hre.run("compile");
    const [deployer] = await hre.ethers.getSigners();

    console.log("Deploying contracts with the account:", deployer.address); // We are printing the address of the deployer
    console.log("Account balance:", (await deployer.getBalance()).toString()); // We are printing the account balance

    const mockBLS = await hre.ethers.getContractFactory("MockBLS");
    const mockBLSContract = await mockBLS.deploy();

    console.log("Waiting for MockBLS deployment...");
    await mockBLSContract.deployed();

    console.log("MockBLS contract address: ", mockBLSContract.address);


    const beaconLightClient = await hre.ethers.getContractFactory("BeaconLightClient");
    const beaconLightClientContract = await beaconLightClient.deploy(
      mockBLSContract.address,
    );

    console.log("waiting for BeaconLightClient deployment...");
    await beaconLightClientContract.deployed();

    console.log(beaconLightClientContract.address);
    console.log("Done!");
  }
);

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.9",
    settings: {
      optimizer: {
        enabled: false,
        runs: 1,
      },
    },
  },
  defaultNetwork: "hardhat",
  networks: {
    hardhat: {
      blockGasLimit: 30000000
    },
    ropsten: {
      url: "https://ropsten.infura.io/v3/2d93e04d5e87490b82e12d9a4bf0c58b",
      accounts: ["7fc217a89873ddcba36225dcdaae0b631c32430bf318e665eefc7cd18dc9b19f"]
    }
  },
  mocha: {
    timeout: 100000000
  }
};
