import yargs from 'yargs';
import fsp from 'fs/promises';
import { BeaconApi } from '../../../relay/implementations/beacon-api';
import config from "../common_config.json";
import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';

(async () => {
  const { ssz } = await import('@lodestar/types');

  const options = yargs.option('slot', {
    type: 'number',
    demandOption: true,
  }).argv;

  const slot = options['slot'];

  const api = new BeaconApi([config['beacon-node']]);

  const storedState = await loadBeaconState(`./states/beacon_state_${slot}.ssz_snappy`);
  const { beaconState } = await api.getBeaconState(slot);

  const changedValidatorIndices: number[] = [];
  const changedValidators = storedState.validators.filter((validator, index) => {
    const hasChanged = hasValidatorChanged(beaconState.validators)({ validator, index });
    if (hasChanged) {
      changedValidatorIndices.push(index);
    }

    return hasChanged;
  });
  console.log(changedValidatorIndices);
  const beforeHash = bytesToHex(ssz.deneb.BeaconState.hashTreeRoot(storedState));
  const afterHash = bytesToHex(ssz.deneb.BeaconState.hashTreeRoot(beaconState));
  console.log(beforeHash);
  console.log(afterHash);
  console.log(changedValidators);
})();

async function loadBeaconState(path: string) {
  const { ssz } = await import('@lodestar/types');
  const bytes = await fsp.readFile(path);
  const beaconState = ssz.deneb.BeaconState.deserialize(bytes);
  return beaconState;
}

function hasValidatorChanged(prevValidators: any[]) {
  return ({ validator, index }: any) =>
    prevValidators[index] === undefined ||
    validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
    validator.slashed !== prevValidators[index].slashed ||
    validator.activationEligibilityEpoch !==
    prevValidators[index].activationEligibilityEpoch ||
    validator.activationEpoch !== prevValidators[index].activationEpoch ||
    validator.exitEpoch !== prevValidators[index].exitEpoch ||
    validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch;
}
