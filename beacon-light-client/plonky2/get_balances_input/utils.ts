import { bytesToHex } from '../../../libs/typescript/ts-utils/bls';
import { Validator, ValidatorPoseidonInput } from '../../../relay/types/types';

export function getZeroValidatorPoseidonInput(): ValidatorPoseidonInput {
  return {
    pubkey: ''.padStart(96, '0'),
    withdrawalCredentials: ''.padStart(64, '0'),
    effectiveBalance: '0',
    slashed: 0,
    activationEligibilityEpoch: '0',
    activationEpoch: '0',
    exitEpoch: '0',
    withdrawableEpoch: '0',
  };
}

export function convertValidatorToValidatorPoseidonInput(
  validator: Validator,
): ValidatorPoseidonInput {
  return {
    pubkey: bytesToHex(validator.pubkey),
    withdrawalCredentials: bytesToHex(validator.withdrawalCredentials),
    effectiveBalance: validator.effectiveBalance.toString(),
    slashed: Number(validator.slashed),
    activationEligibilityEpoch: validator.activationEligibilityEpoch.toString(),
    activationEpoch: validator.activationEpoch.toString(),
    exitEpoch:
      validator.exitEpoch === Infinity
        ? (2n ** 64n - 1n).toString()
        : validator.exitEpoch.toString(),
    withdrawableEpoch:
      validator.withdrawableEpoch === Infinity
        ? (2n ** 64n - 1n).toString()
        : validator.withdrawableEpoch.toString(),
  };
}
