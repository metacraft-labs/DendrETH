import { getBeaconApi } from '@dendreth/relay/implementations/beacon-api';
import common_config from '../common_config.json';
import { Tree } from '@chainsafe/persistent-merkle-tree/lib/tree';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';
import { CommandLineOptionsBuilder } from './utils/cmdline';

(async function() {
  const args = new CommandLineOptionsBuilder()
    .option('slot', {
      type: 'number',
      demandOption: true,
    })
    .build();

  const { ssz } = await import('@lodestar/types');
  const api = await getBeaconApi(common_config['beacon-node']);

  const { beaconState } = (await api.getBeaconState(args['slot']))!;
  const validators = beaconState.validators;

  const validatorsViewDU = ssz.deneb.BeaconState.fields.validators.toViewDU(validators);

  const tree = new Tree(validatorsViewDU.node);
  console.log(
    'validators root',
    bytesToHex(tree.getRoot(1n)),
  );
  console.log(
    'proof root',
    bytesToHex(tree.getRoot(2n)),
  );
  console.log('validators length', validators.length)
})();
