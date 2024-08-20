import { bigint_to_array, formatHex, hexToBytes } from '@dendreth/utils/ts-utils/bls';
import { reverseEndianness } from '@dendreth/utils/ts-utils/hex-utils';
import { PointG1 } from '@noble/bls12-381';
import { buildPoseidonReference } from 'circomlibjs';

export async function getPoseidonInputs(pubkeys: string[]) {
  let pubkeyPoints = pubkeys.map(x => PointG1.fromHex(formatHex(x)).toAffine());
  let pubkeyArrays = pubkeyPoints.map(x => [
    bigint_to_array(55, 7, x[0].value),
    bigint_to_array(55, 7, x[1].value),
  ]);

  let poseidon = await buildPoseidonReference();

  let poseidonValFlat: string[] = [];
  for (let i = 0; i < 512; i++) {
    for (let j = 0; j < 7; j++)
      for (let l = 0; l < 2; l++) {
        poseidonValFlat[i * 7 * 2 + j * 2 + l] = pubkeyArrays[i][l][j];
      }
  }

  let prev: any = 0;

  const LENGTH = 512 * 2 * 7;
  const NUM_ROUNDS = LENGTH / 16;
  for (let i = 0; i < NUM_ROUNDS; i++) {
    let inputs: any[] = [];
    for (let j = 0; j < 16; j++) {
      inputs.push(poseidonValFlat[i * 16 + j]);
    }
    if (i < NUM_ROUNDS - 1) {
      prev = poseidon(inputs, prev, 1);
    } else {
      prev = poseidon(inputs, prev, 2);
    }
  }

  return poseidon.F.toString(prev[1]);
}

export function toLittleEndianBytes(value: bigint): Uint8Array {
  return hexToBytes(
    reverseEndianness(
      BigInt(value).toString(16).padStart(64, '0'),
    ),
  )
}
