import * as path from "path";
import { ethers } from "hardhat";
import { getFilesInDir, getProofInput } from "./utils";
import { formatJSONUpdate, hashTreeRootSyncCommitee, JSONUpdate } from "./utils/format";
import * as  constants from "./utils/constants";
import { groth16 } from "snarkjs";
import { readFileSync, writeFileSync } from "fs";
import { exec } from "child_process";
import { promisify } from "util";

const promiseExec = promisify(exec);

// TODO: fix
describe("BeaconLightClient", async function () {
  let NETWORK;
  let SNAPSHOT;
  let UPDATES: JSONUpdate[];
  let blc;

  beforeEach(async function () {
    NETWORK = 'mainnet';

    SNAPSHOT = require(`../../data/${NETWORK}/snapshot.json`).data.v;

    UPDATES = getFilesInDir(
      path.join(__dirname, "..", "..", "data", NETWORK, "updates")
    ).map(u => formatJSONUpdate(JSON.parse(u.toString()), constants.GENESIS_FORK_VERSION.join("")));
  });

  beforeEach(async function () {
    blc = await (await ethers.getContractFactory("BeaconLightClient")).deploy(
      UPDATES[0].attested_header.slot,
      UPDATES[0].attested_header.proposer_index,
      UPDATES[0].attested_header.parent_root,
      UPDATES[0].attested_header.body_root,
      UPDATES[0].attested_header.state_root,
      hashTreeRootSyncCommitee(UPDATES[0].next_sync_committee), // sync committee hash
      constants.GENESIS_VALIDATORS_ROOT
    );
  });

  it("Importing real data", async function () {
    console.log(" >>> Begin importing of real updates");
    let period = 291;
    let prevUpdate = UPDATES[0];
    for (let update of UPDATES.slice(1)) {
      writeFileSync("input.json", JSON.stringify(await getProofInput(prevUpdate, update)));

      console.log("Witness generation...");
      console.log((await promiseExec("../circom/build/god_please/proof_efficient/proof_efficient_cpp/proof_efficient input.json witness.wtns")).stdout);

      console.log("Proof generation...");
      console.log((await promiseExec(`../../vendor/rapidsnark/build/prover ../circom/build/god_please/proof_efficient/proof_efficient_0.zkey witness.wtns data/${NETWORK}/proof${period}.json data/${NETWORK}/public${period}.json`)).stdout);

      const proof = JSON.parse(readFileSync(`data/${NETWORK}/proof${period}.json`).toString());
      const publicSignals = JSON.parse(readFileSync(`data/${NETWORK}/public${period}.json`).toString());
      const calldata = await groth16.exportSolidityCallData(proof, publicSignals);

      const argv = calldata.replace(/["[\]\s]/g, "").split(',').map(x => BigInt(x).toString());

      const a = [argv[0], argv[1]];
      const b = [[argv[2], argv[3]], [argv[4], argv[5]]];
      const c = [argv[6], argv[7]];

      const lightClientUpdate = {
        attested_header: update.attested_header,
        finalized_header: update.finalized_header,
        finality_branch: update.finality_branch,
        sync_aggregate: update.sync_aggregate,
        signature_slot: update.signature_slot,
        fork_version: constants.ALTAIR_FORK_VERSION,
        next_sync_committee_root: hashTreeRootSyncCommitee(update.next_sync_committee),
        next_sync_committee_branch: update.next_sync_committee_branch,
        a: a,
        b: b,
        c: c
      };

      await promiseExec("rm input.json witness.wtns")

      console.log(` >>> Importing update for period ${period}...`);
      const transaction = await blc.light_client_update(lightClientUpdate, { gasLimit: 30000000 });
      const result = await transaction.wait();
      console.log(` >>> Successfully imported update for period ${period++}!`);

      prevUpdate = update;
    }
  });
});
