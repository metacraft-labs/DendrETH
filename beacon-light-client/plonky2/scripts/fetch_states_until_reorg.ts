import fs from 'fs';
import fsp from 'fs/promises';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import config from "../common_config.json";
import { sleep } from '../../../libs/typescript/ts-utils/common-utils';

(async () => {
  const api = new BeaconApi([config['beacon-node']]);

  const es = await api.subscribeForEvents(['head', 'chain_reorg']);
  es.on('head', async event => {
    const slot = JSON.parse(event.data)['slot'];
    const beaconStateBytes = await api.getBeaconStateSSZBytes(slot);
    await saveBeaconState(beaconStateBytes, slot);

    const path = `./states/beacon_staste_${slot - 100}.ssz_snappy`;
    if (fs.existsSync(path)) {
      await fsp.unlink(path);
    }
  });
  es.on('chain_reorg', async event => {
    console.log('chain reorg happenned')
    console.log(event);
    await sleep(240000);
    process.exit(0);
  });
})();

async function saveBeaconState(beaconState: Uint8Array, slotId: number) {
  if (!fs.existsSync('states')) {
    await fsp.mkdir('states');
  }
  fsp.writeFile(`./states/beacon_state_${slotId}.ssz_snappy`, beaconState)
}
