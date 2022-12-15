import { ssz } from '@chainsafe/lodestar-types';
import { ByteVectorType, UintNumberType, BooleanType } from '@chainsafe/ssz';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { PointG1 } from '@noble/bls12-381';
import { writeFile } from 'fs/promises';
import {
  bigint_to_array,
  bytesToHex,
} from '../../../../libs/typescript/ts-utils/bls';

export default async function getInput(
  validators: ValueOfFields<{
    pubkey: ByteVectorType;
    withdrawalCredentials: ByteVectorType;
    effectiveBalance: UintNumberType;
    slashed: BooleanType;
    activationEligibilityEpoch: UintNumberType;
    activationEpoch: UintNumberType;
    exitEpoch: UintNumberType;
    withdrawableEpoch: UintNumberType;
  }>[],
  index: number,
  epoch: number,
) {
  const withdrawCredentials: string[][] = [];
  const effectiveBalance: string[][] = [];
  const slashed: string[] = [];
  const activationEligibilityEpoch: string[] = [];
  const activationEpoch: string[] = [];
  const exitEpoch: string[] = [];
  const withdrawableEpoch: string[][] = [];

  for (let i = 0; i < validators.length; i++) {
    const validatorTree = new Tree(
      ssz.phase0.Validator.toViewDU(validators[i]).node,
    );

    withdrawCredentials.push(
      BigInt('0x' + bytesToHex(validatorTree.getNode(9n).root))
        .toString(2)
        .padStart(256, '0')
        .split(''),
    );

    effectiveBalance.push(
      BigInt('0x' + bytesToHex(validatorTree.getNode(10n).root))
        .toString(2)
        .padStart(256, '0')
        .split(''),
    );

    if(validators[i].slashed) {
      console.log("WTF", i);
    }

    slashed.push(Number(validators[i].slashed).toString());

    activationEligibilityEpoch.push(
      validators[i].activationEligibilityEpoch.toString(),
    );

    activationEpoch.push(validators[i].activationEpoch.toString());

    exitEpoch.push(
      validators[i].exitEpoch.toString() === 'Infinity'
        ? '18446744073709551615'
        : validators[i].exitEpoch.toString(),
    );

    withdrawableEpoch.push(
      BigInt('0x' + bytesToHex(validatorTree.getNode(15n).root))
        .toString(2)
        .padStart(256, '0')
        .split(''),
    );
  }

  let input = {
    points: validators.map(x => [
      bigint_to_array(55, 7, PointG1.fromHex(x.pubkey).toAffine()[0].value),
      bigint_to_array(55, 7, PointG1.fromHex(x.pubkey).toAffine()[1].value),
    ]),
    zero: [...[...Array(64).keys()].map(() => 1)],
    withdrawCredentials,
    effectiveBalance,
    slashed,
    activationEligibilityEpoch,
    activationEpoch,
    exitEpoch,
    withdrawableEpoch,
    bitmask: validators.map(x =>
      Number(
        x.exitEpoch > epoch &&
          !x.slashed &&
          x.activationEpoch < epoch &&
          x.activationEligibilityEpoch < epoch,
      ),
    ),
    currentEpoch: epoch,
  };

  await writeFile(
    `inputs_first_level/input${index}.json`,
    JSON.stringify(input),
  );
}
