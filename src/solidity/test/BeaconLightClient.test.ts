import * as path from "path";
import { ethers } from "hardhat";
import { getFilesInDir, getProofInput } from "./utils";
import { formatJSONUpdate, hashTreeRootSyncCommitee } from "./utils/format";
import * as  constants from "./utils/constants";
import { exec } from "child_process";
import { groth16 } from "snarkjs";
import { readFileSync, writeFileSync } from "fs";

// TODO: fix
describe.skip("BeaconLightClient", async function () {
  let NETWORK;
  let SNAPSHOT;
  let UPDATES;
  let MOCK_BLS;
  let blc;

  beforeEach(async function () {
    NETWORK = 'mainnet';

    SNAPSHOT = require(`../../data/${NETWORK}/snapshot.json`).data.v;

    UPDATES = getFilesInDir(
      path.join(__dirname, "..", "..", "data", NETWORK, "updates")
    ).map(u => formatJSONUpdate(JSON.parse(u.toString()), constants.GENESIS_FORK_VERSION.join("")));

    MOCK_BLS = await (await ethers.getContractFactory("MockBLS")).deploy();
  });

  beforeEach(async function () {
    blc = await (await ethers.getContractFactory("BeaconLightClient")).deploy(
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
    let prevUpdate = UPDATES[0];
    for (let update of UPDATES.slice(1)) {
      writeFileSync("input.json", JSON.stringify(await getProofInput(prevUpdate, update)));

      await exec("../circom/build/proof_efficient/proof_efficient_cpp/proof_efficient input.json witness.wtns");

      await exec("../../vendor/rapidsnark/build/prover ../circom/build/proof_efficient/proof_efficient_0.zkey ../circom/build/proof_efficient/proof_efficient_cpp/witness.wtns proof.json public.json");

      const proof = JSON.parse(readFileSync("proof.json").toString());
      const publicSignals = JSON.parse(readFileSync("public.json").toString());
      const calldata = await groth16.exportSolidityCallData(proof, publicSignals);

      const argv = calldata.replace(/["[\]\s]/g, "").split(',').map(x => BigInt(x).toString());

      const a = [argv[0], argv[1]];
      const b = [[argv[2], argv[3]], [argv[4], argv[5]]];
      const c = [argv[6], argv[7]];

      await exec("rm input.json witness.wtns proof.json public.json");

      const lightClientUpdate = {
        attested_header: update.attested_header,
        finalized_header: update.finalized_header,
        finality_branch: update.finality_branch,
        sync_aggregate: update.sync_aggregate,
        signature_slot: update.signature_slot,
        fork_version: constants.ALTAIR_FORK_VERSION,
        next_sync_committee_root: hashTreeRootSyncCommitee(update.next_sync_committee),
        next_sync_committee_branch: update.next_sync_committee_branch,
        proof: { a, b, c }
      };

      console.log(` >>> Importing update for period ${period}...`);
      await blc.light_client_update(lightClientUpdate, { gasLimit: 30000000 });
      console.log(` >>> Successfully imported update for period ${period++}!`);

      prevUpdate = update;
    }
  });
});
