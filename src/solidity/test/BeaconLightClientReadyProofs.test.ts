import * as path from "path";
import { ethers } from "hardhat";
import { getFilesInDir, getSolidityProof } from "./utils";
import { FormatedJsonUpdate, formatJSONUpdate, formatLightClientUpdate } from "./utils/format";
import * as  constants from "./utils/constants";
import { getConstructorArgs } from "../tasks/utils";

const NETWORK = 'mainnet';

describe.only("BeaconLightClientReadyProofs", async function () {
  let UPDATES: FormatedJsonUpdate[];
  let blc;

  beforeEach(async function () {
    UPDATES = getFilesInDir(
      path.join(__dirname, "..", "..", "..", "vendor", "eth2-light-client-updates", NETWORK, "updates")
    ).map(u => formatJSONUpdate(JSON.parse(u.toString()), constants.GENESIS_FORK_VERSION.join("")));
  });

  beforeEach(async function () {
    blc = await (await ethers.getContractFactory("BeaconLightClient")).deploy(...getConstructorArgs(NETWORK));
  });

  it("Importing real data", async function () {
    console.log(" >>> Begin importing of real updates");
    let period = 291;
    let prevUpdate = UPDATES[0];
    for (let update of UPDATES.slice(1, 4)) {
      const proof = await getSolidityProof(prevUpdate, update, NETWORK);
      const lightClientUpdate = formatLightClientUpdate(update, proof);

      console.log(` >>> Importing update for period ${period}...`);
      const transaction = await blc.light_client_update(lightClientUpdate, { gasLimit: 30000000 });
      const result = await transaction.wait();
      console.log(` >>> Successfully imported update for period ${period++}!`);

      prevUpdate = update;
    }
  });
});
