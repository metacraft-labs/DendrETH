const axios = require("axios").default;
const { writeFile } = require("fs");
// The correct path should be selected, where the snapshot.json is, in order to get the data
const bootstrap = require("../../../eth2-light-client-updates/mainnet/bootstrap.json");

/*
The HOST should stay "http://localhost", while the PORT may change, depending on how the beacon node has been set up
The HOST is at PORT = 5053, because it has been started with: 
"NODE_ID=1 ./run-mainnet-beacon-node.sh --light-client-data-serve --light-client-data-import-mode=on-demand --light-client-data-max-periods=1000", 
thus it is not 5052, but 5053. If the node is set without the NODE_ID parameter, than it's going to be 5052
The PATH may also change depending on how data is read from the snapshot.json file and written to the updates folder
One sync committee period is around 27 hours -  EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH, where one slot is 12s and one epoch is 32 slots
START_PERIOD - first committee after the Altair hard fork
*/
const HOST = "http://localhost";
const port = 5053;
const path = "../../../eth2-light-client-updates/mainnet";
const EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
const SLOTS_PER_EPOCH = 32;

async function getBlockData() {
  // The latest finanlized header is used to determine the sync committee period
  const latestFinalizedHeader = await axios.get(
    `${HOST}:${port}/eth/v1/beacon/headers/finalized`
  );

  const slot = bootstrap.data.v.header.slot;
  const current_sync_committee_period = Math.floor(
    slot / (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH)
  );
  const endPoint = Math.ceil(
    latestFinalizedHeader.data.data.header.message.slot /
      (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH)
  );
  // Determing the start and end point for the updates
  const count = endPoint - current_sync_committee_period;

  const updatesResponse = await axios.get(
    `${HOST}:${port}/eth/v0/beacon/light_client/updates?start_period=${current_sync_committee_period}&count=${count}`
  );

  // Updating the prater/updates folder
  for (let i = 0; i < updatesResponse.data.data.length; i++) {
    writeFile(
      `${path}/updates/00${current_sync_committee_period + i}.json`,
      JSON.stringify(updatesResponse.data.data[i], null, 2),
      { flag: "w+" },
      (err) => {
        if (err) throw err;
        console.log(`Update №${current_sync_committee_period + i} has been saved!`);
      }
    );
  }

  // Updating the snapshot.json file for the next time, the update folder should be updated, in order to do not start from the very start
  const snapshotData = await axios.get(
    `${HOST}:${port}/eth/v0/beacon/light_client/bootstrap/${latestFinalizedHeader.data.data.root}`
  );

  writeFile(
    `${path}/snapshot.json`,
    JSON.stringify(snapshotData.data, null, 2),
    { flag: "w+" },
    (err) => {
      if (err) throw err;
      console.log("Bootstrap has been updated and saved!");
    }
  );
}

getBlockData();
