import { hexToBytes } from '@dendreth/utils/ts-utils/bls';
import { Validator } from '../types/types';

function stringToNumber(str: string): number {
  return str == BigInt(2n ** 64n - 1n).toString()
    ? Infinity
    : Number(str);
}

export function validatorFromValidatorJSON(
  json: any,
): Validator {
  return {
    pubkey: hexToBytes(json.pubkey),
    withdrawalCredentials: hexToBytes(json.withdrawalCredentials),
    exitEpoch: stringToNumber(json.exitEpoch),
    activationEpoch: stringToNumber(json.activationEpoch),
    effectiveBalance: stringToNumber(json.effectiveBalance),
    withdrawableEpoch: stringToNumber(json.withdrawableEpoch),
    activationEligibilityEpoch: stringToNumber(json.activationEligibilityEpoch),
    slashed: json.slashed,
  };
}
