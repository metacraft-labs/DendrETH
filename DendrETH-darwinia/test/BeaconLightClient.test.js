const { expect } = require('chai');
const { ethers } = require("hardhat");
const path = require("path");
const { getFilesInDir } = require("./utils");
// const fs = require("fs");
// // const blcABIRaw = fs.readFileSync("./artifacts/contracts/bridge/src/truth/eth/BeaconLightClient.sol/BeaconLightClient.json");
// // const blcABI = JSON.parse(blcABIRaw);
const formatUpdate = (update, FORK_VERSION) => {
  update = JSON.parse(update);
  update.sync_aggregate.sync_committee_bits =
    update.sync_aggregate.sync_committee_bits.replace("0x", "");
  update.sync_aggregate.sync_committee_bits = [
    "0x".concat(
      update.sync_aggregate.sync_committee_bits.slice(
        0,
        update.sync_aggregate.sync_committee_bits.length / 2
      )
    ),
    "0x".concat(
      update.sync_aggregate.sync_committee_bits.slice(
        update.sync_aggregate.sync_committee_bits.length / 2
      )
    ),
  ];

  update.fork_version = FORK_VERSION;

  return update;
};

// // (async () => {
// //   const network = 'mainnet';
// //   const UPDATES = getFilesInDir(
// //     path.join(__dirname, "data", network, "updates")
// //   ).map(u => formatUpdate(u, "0x00000000"));
// //   const SNAPSHOT = require(`./data/${network}/snapshot.json`).data.v;

// //   let prev_sync_committee = SNAPSHOT.current_sync_committee;

// //   const [account] = await ethers.getSigners();

// //   const blc = new ethers.Contract(
// //     "0x9B77E0Ea1E309c6317B9dec137184a75d009a717",
// //     blcABI.abi,
// //     account
// //   );

// //   console.log((await blc.state_root()));


// //   for (let update of UPDATES.slice(0, 1)) {
// //     update.prev_sync_committee = prev_sync_committee;
// //     // let interface = new ethers.utils.Interface(blcABI.abi);
// //     // const result = interface.encodeFunctionData('light_client_update', [update]);
// //     // console.log(result);
// //     await blc.light_client_update(update, {
// //       gasLimit: 30000000,

// //     });
// //     const state_root = await blc.state_root();
// //     console.log(state_root);
// //     prev_sync_committee = update.next_sync_committee;
// //   }
// // })();



describe("BeaconLightClient", async function () {


  it("mainent", async function () {
    const network = 'mainnet';
    const FORK_VERSION = "0x00000000";
    const GENESIS_VALIDATORS_ROOT = "0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95"
    const SNAPSHOT = require(`./data/${network}/snapshot.json`).data.v;
    const bls = await (await ethers.getContractFactory("MockBLS")).deploy();
    const blc = await (
      await ethers.getContractFactory("BeaconLightClient")
    ).deploy(
      bls.address,
      SNAPSHOT.header.slot,
      SNAPSHOT.header.proposer_index,
      SNAPSHOT.header.parent_root,
      SNAPSHOT.header.state_root,
      SNAPSHOT.header.body_root,
      "0x52bbd8287d0e455ce6cd732fa8a5f003e2ad82fd0ed3a59516f9ae1642f1b182",
      GENESIS_VALIDATORS_ROOT
    );

    const UPDATES = getFilesInDir(
      path.join(__dirname, "data", network, "updates")
    ).map(u => formatUpdate(u, FORK_VERSION));

    let prev_sync_committee = SNAPSHOT.current_sync_committee;

    for (let update of UPDATES.slice(0, 5)) {
      update.signature_sync_committee = prev_sync_committee;
      await blc.import_finalized_header(update, {
        gasLimit: 30000000,
      });

      await blc.import_next_sync_committee(update, {gasLimit: 30000000});

      const state_root = await blc.state_root();
      expect(state_root).to.equal(update.finalized_header.state_root);
      console.log(state_root);
      prev_sync_committee = update.next_sync_committee;
    }
  });

  it("prater", async function () {
    const network = 'prater';
    const FORK_VERSION = "0x00001020";
    const GENESIS_VALIDATORS_ROOT = "0x043db0d9a83813551ee2f33450d23797757d430911a9320530ad8a0eabc43efb"
    const SNAPSHOT = require(`./data/${network}/snapshot.json`).data.v;
    const bls = await (await ethers.getContractFactory("MockBLS")).deploy();
    const blc = await (
      await ethers.getContractFactory("BeaconLightClient")
    ).deploy(
      bls.address,
      SNAPSHOT.header.slot,
      SNAPSHOT.header.proposer_index,
      SNAPSHOT.header.parent_root,
      SNAPSHOT.header.state_root,
      SNAPSHOT.header.body_root,
      "0xa346acdfb26fa13397eb6e988b28a3a52146648db4bb230dca5d9261d300d4f9",
      GENESIS_VALIDATORS_ROOT
    );

    const UPDATES = getFilesInDir(
      path.join(__dirname, "data", network, "updates")
    ).map(u => formatUpdate(u, FORK_VERSION));

    let prev_sync_committee = SNAPSHOT.current_sync_committee;

    for (let update of UPDATES.slice(0, 5)) {
      update.signature_sync_committee = prev_sync_committee;
      await blc.import_finalized_header(update, {
        gasLimit: 30000000,
      });

      await blc.import_next_sync_committee(update, {gasLimit: 30000000});

      const state_root = await blc.state_root();
      expect(state_root).to.equal(update.finalized_header.state_root);
      console.log(state_root);
      prev_sync_committee = update.next_sync_committee;
    }
  });
});
