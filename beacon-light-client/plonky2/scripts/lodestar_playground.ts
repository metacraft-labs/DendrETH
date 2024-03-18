import config from "../common_config.json";
import yargs from 'yargs';
import { BeaconApi } from "../../../relay/implementations/beacon-api";
import { bytesToHex } from "../../../libs/typescript/ts-utils/bls";

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs.option('slot', {
    type: 'number',
    demandOption: true,
  }).argv;

  const offset = 25880;
  const take = 19;

  const slot = options['slot'];
  const api = new BeaconApi([config['beacon-node']]);
  const { beaconState } = await api.getBeaconState(slot);
  beaconState.validators = beaconState.validators.slice(offset, offset + take);
  const hash = ssz.deneb.BeaconState.fields.validators.hashTreeRoot(beaconState.validators);
  console.log(bytesToHex(hash));
  console.log(beaconState.validators.map(validator => bytesToHex(validator.pubkey)));
})();
