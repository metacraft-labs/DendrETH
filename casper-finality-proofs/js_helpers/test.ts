import { sha256 } from 'ethers/lib/utils';
import { BeaconApi } from '../../relay/implementations/beacon-api';
import { bytesToHex } from '../../libs/typescript/ts-utils/bls';

// let result = sha256(
//   '0x12300000000000000000000000000000000000000000000000000000000000001230000000000000000000000000000000000000000000000000000000000000',
// );

// console.log(result);
(async () => {
  const beaconApi = new BeaconApi([
    'http://unstable.mainnet.beacon-api.nimbus.team',
  ]);
  const validators = await beaconApi.getValidators(6953401);
  const { ssz } = await import('@lodestar/types');
  console.log(bytesToHex(validators[0].pubkey));
  console.log(
    'pubkey hash',
     bytesToHex(ssz.phase0.Validator.fields.pubkey.hashTreeRoot(validators[0].pubkey)),
  );
  console.log(bytesToHex(validators[0].withdrawalCredentials));
  console.log(bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[0])));
})();
