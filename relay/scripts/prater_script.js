const axios = require("axios").default;
const { writeFile } = require("fs");
// The correct path should be selected, where the snapshot.json is, in order to get the data
const data = require("../eth2-light-client-updates/prater/snapshot.json");

/*
The HOST should stay "http://localhost", while the PORT may change, depending on how the beacon node has been set up
The PATH may also change depending on how data is read from the snapshot.json file and written to the updates folder
One sync committee period is around 27 hours -  EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH, where one slot is 12s and one epoch is 32 slots
START_PERIOD - first committee after the Altair hard fork(The Altair hard fork was done during the 143th sync committee period)
*/
const HOST = "http://localhost";
const PORT = 5052;
const PATH = "../eth2-light-client-updates/prater";
const EPOCHS_PER_SYNC_COMMITTEE_PERIOD = 256;
const SLOTS_PER_EPOCH = 32;
const START_PERIOD = 144;

const snapshot = parseInt(data.snapshot);

async function getBlockData() {
  // The latest finanlized header is used to determine the sync committee period
  const latestFinalizedHeader = await axios.get(
    `${HOST}:${PORT}/eth/v1/beacon/headers/finalized`
  );

  const slot = latestFinalizedHeader.data.data.header.message.slot;
  const sync_committee_period =
    slot / (EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH);

  // Determing the start and end point for the updates
  const starting_point = START_PERIOD + snapshot;
  const count = Math.floor(sync_committee_period) - starting_point + 1;

  const updatesResponse = await axios.get(
    `${HOST}:${PORT}/eth/v0/beacon/light_client/updates?start_period=${starting_point}&count=${count}`
  );

  // Updating the prater/updates folder
  for (let i = 0; i < updatesResponse.data.data.length; i++) {
    writeFile(
      `${PATH}/updates/00${starting_point + i}.json`,
      JSON.stringify(updatesResponse.data.data[i], null, "\t"),
      { flag: "w+" },
      (err) => {
        if (err) throw err;
        console.log("updatesResponse data has been saved!");
      }
    );
  }

  // Updating the snapshot.json file for the next time, the update folder should be updated, in order to do not start from the very start
  writeFile(
    `${PATH}/snapshot.json`,
    JSON.stringify({ snapshot: `${snapshot + count}` }, null, "\t"),
    { flag: "w+" },
    (err) => {
      if (err) throw err;
      console.log("Snapshot count has been updated and saved!");
    }
  );
}

getBlockData();
