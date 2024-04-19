import { hexToBytes } from '@dendreth/utils/ts-utils/bls';
import { Validator, ValidatorShaInput } from '../types/types';

export function validatorFromValidatorJSON(
  json: ValidatorShaInput,
  ssz: any,
): Validator {
  const validator: Validator = {
    pubkey: hexToBytes(json.pubkey),
    withdrawalCredentials: hexToBytes(json.withdrawalCredentials),
    effectiveBalance: ssz.phase0.Validator.fields.effectiveBalance.deserialize(
      hexToBytes(json.effectiveBalance).slice(0, 8),
    ),
    slashed: ssz.phase0.Validator.fields.slashed.deserialize(
      hexToBytes(json.slashed).slice(0, 1),
    ),
    activationEligibilityEpoch:
      ssz.phase0.Validator.fields.activationEligibilityEpoch.deserialize(
        hexToBytes(json.activationEligibilityEpoch).slice(0, 8),
      ),
    activationEpoch: ssz.phase0.Validator.fields.activationEpoch.deserialize(
      hexToBytes(json.activationEpoch).slice(0, 8),
    ),
    exitEpoch: ssz.phase0.Validator.fields.exitEpoch.deserialize(
      hexToBytes(json.exitEpoch).slice(0, 8),
    ),
    withdrawableEpoch:
      ssz.phase0.Validator.fields.withdrawableEpoch.deserialize(
        hexToBytes(json.withdrawableEpoch).slice(0, 8),
      ),
  };

  return validator;
}
