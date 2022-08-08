// const { expect } = require("chai");
const { ethers } = require("hardhat");

const path = require("path");

const { getFilesInDir } = require("./utils");

SNAPSHOT = require("./data/mainnet/snapshot.json").data.v;

describe("BeaconLightClient", async function () {
  const formatUpdate = (update) => {
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

    return update;
  };

  const UPDATES = getFilesInDir(
    path.join(__dirname, "data", "mainnet", "updates")
  ).map(formatUpdate);
  const FORK_VERSION = "0x02000000";

  beforeEach(async function () {
    bls = await (await ethers.getContractFactory("BLS")).deploy();

    blc = await (
      await ethers.getContractFactory("BeaconLightClient")
    ).deploy(
      bls.address,
      SNAPSHOT.header.slot,
      SNAPSHOT.header.proposer_index,
      SNAPSHOT.header.parent_root,
      SNAPSHOT.header.body_root,
      SNAPSHOT.header.state_root,
      "0x52bbd8287d0e455ce6cd732fa8a5f003e2ad82fd0ed3a59516f9ae1642f1b182", // hash_tree_root(SNAPSHOT.current_sync_committee),
      "0x32251a5a748672e3acb1e574ec27caf3b6be68d581c44c402eb166d71a89808e" // GENESIS_VALIDATORS_ROOT
    );
  });

  it("test1", async function () {
    for (let update of UPDATES) {
      let syncCommitteePeriodUpdate = {
        next_sync_committee: update.next_sync_committee,
        next_sync_committee_branch: update.next_sync_committee_branch,
      };

      let finalizedHeaderUpdate = {
        attested_header: update.attested_header,
        signature_sync_committee: update.next_sync_committee,
        finalized_header: update.finalized_header,
        finality_branch: update.finality_branch,
        sync_aggregate: update.sync_aggregate,
        fork_version: FORK_VERSION,
        signature_slot: update.signature_slot,
      };

      await blc.import_finalized_header(finalizedHeaderUpdate, {
        gasLimit: 30000000,
      });
      await blc.import_next_sync_committee(syncCommitteePeriodUpdate, {
        gasLimit: 30000000,
      });
    }
  });
});
