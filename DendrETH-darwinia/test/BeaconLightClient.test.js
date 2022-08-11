const { expect } = require('chai');
const { ethers } = require("hardhat");

const path = require("path");

const { getFilesInDir } = require("./utils");

describe("BeaconLightClient", async function () {
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
      SNAPSHOT.current_sync_committee,
      SNAPSHOT.header,
      GENESIS_VALIDATORS_ROOT
    );

    const UPDATES = getFilesInDir(
      path.join(__dirname, "data", network, "updates")
    ).map(u => formatUpdate(u, FORK_VERSION));

    let prev_sync_committee = SNAPSHOT.current_sync_committee;

    for (let update of UPDATES.slice(0, 5)) {
      update.prev_sync_committee = prev_sync_committee;
      await blc.light_client_update(update, {
        gasLimit: 30000000,
      });

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
      SNAPSHOT.current_sync_committee,
      SNAPSHOT.header,
      GENESIS_VALIDATORS_ROOT
    );

    const UPDATES = getFilesInDir(
      path.join(__dirname, "data", network, "updates")
    ).map(u => formatUpdate(u, FORK_VERSION));

    let prev_sync_committee = SNAPSHOT.current_sync_committee;

    for (let update of UPDATES.slice(0, 5)) {
      update.prev_sync_committee = prev_sync_committee;
      await blc.light_client_update(update, {
        gasLimit: 30000000,
      });

      const state_root = await blc.state_root();
      expect(state_root).to.equal(update.finalized_header.state_root);
      console.log(state_root);
      prev_sync_committee = update.next_sync_committee;
    }
  });
});
