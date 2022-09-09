import * as path from "path";
import { expect } from 'chai';
import { ethers } from "hardhat";
import { getFilesInDir } from "./utils";
import { formatJSONUpdate } from "./utils/format";
import * as  constants from "./utils/constants";

// TODO: fix
describe.skip("BeaconLightClient", async function () {
    let NETWORK;
    let SNAPSHOT;
    let UPDATES;
    let MOCK_BLS;
    let blc;

    before(async function () {
        NETWORK = 'mainnet';

        SNAPSHOT = require(`../../data/${NETWORK}/snapshot.json`).data.v;

        UPDATES = getFilesInDir(
            path.join(__dirname, "..", "..", "data", NETWORK, "updates")
        ).map(u => formatJSONUpdate(JSON.parse(u), constants.GENESIS_FORK_VERSION.join("")));

        MOCK_BLS = await (await ethers.getContractFactory("MockBLS")).deploy();
    });

    beforeEach(async function () {
        blc = await (await ethers.getContractFactory("BeaconLightClient")).deploy(
            MOCK_BLS.address,
            SNAPSHOT.header.slot,
            SNAPSHOT.header.proposer_index,
            SNAPSHOT.header.parent_root,
            SNAPSHOT.header.body_root,
            SNAPSHOT.header.state_root,
            "0x52bbd8287d0e455ce6cd732fa8a5f003e2ad82fd0ed3a59516f9ae1642f1b182", // sync committee hash
            constants.GENESIS_VALIDATORS_ROOT
        );
    });

    it("Importing real data", async function () {
        console.log(" >>> Begin importing of real updates");
        let period = 291;
        for (let update of UPDATES) {
            const syncCommitteePeriodUpdate = {
                next_sync_committee: update.next_sync_committee,
                next_sync_committee_branch: update.next_sync_committee_branch
            };

            const finalizedHeaderUpdate = {
                attested_header: update.attested_header,
                signature_sync_committee: update.next_sync_committee,
                finalized_header: update.finalized_header,
                finality_branch: update.finality_branch,
                sync_aggregate: update.sync_aggregate,
                signature_slot: update.signature_slot,
                fork_version: constants.ALTAIR_FORK_VERSION
            };

            console.log(` >>> Importing update for period ${period}...`);
            console.log(syncCommitteePeriodUpdate);
            await blc.import_next_sync_committee(syncCommitteePeriodUpdate, { gasLimit: 30000000 });
            await blc.import_finalized_header(finalizedHeaderUpdate, { gasLimit: 30000000 });
            console.log(` >>> Successfully imported update for period ${period++}!`);

            // let prev_sync_committee = SNAPSHOT.current_sync_committee;

            // for (let update of UPDATES.slice(0, 1)) {
            //     update.signature_sync_committee = prev_sync_committee;
            //     const result = await MockBLS.hashToField("0x505e873586be492495799d1e47b61720d9a0a70dca4a6bb661127e9207687049");

            //     console.log(result[0][0]);
            //     console.log(result[0][1]);
            //     console.log(result[1][0]);
            //     console.log(result[1][1]);

            //     const state_root = await blc.state_root();
            //     expect(state_root).to.equal(update.finalized_header.state_root);
            //     console.log(state_root);
            //     prev_sync_committee = update.next_sync_committee;
            // }
        }
    });
});
