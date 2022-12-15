import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { PointG1 } from '@noble/bls12-381';
import { exec as _exec } from 'child_process';
import { readFileSync, rmSync } from 'fs';
import { promisify } from 'util';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '../../../../libs/typescript/ts-utils/bls';
import getInput from './get-aggregate-pubkeys-input';
import { sha256 } from 'ethers/lib/utils';
import path from 'path';
import { rm, writeFile } from 'fs/promises';

const exec = promisify(_exec);

const SIZE = 128;

let validatorsJSON = JSON.parse(
  readFileSync('../../../../validators.json', 'utf-8'),
);

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

let beaconStateJson = JSON.parse(
  readFileSync('../../../../beacon_state.json', 'utf-8'),
).data;

(async () => {
  for (let i = 0; i < SIZE / 64; i++) {
    await getInput(
      validators.slice(i * 64, i * 64 + 64),
      i,
      Math.floor(beaconStateJson.slot / 32),
    );
  }
  if (SIZE % 64 != 0) {
    await getInput(
      validators.slice((SIZE / 64) * 64, (SIZE / 64) * 64 + (SIZE % 64)),
      SIZE / 64,
      Math.floor(beaconStateJson.slot / 32),
    );
  }

  // async function calculateWitness(
  //   witnessCalculatorPath: string,
  //   inputDir: string,
  //   witnessDir: string,
  //   numberOfCalculations: number,
  // ) {
  //   let promises: Promise<void>[] = [];
  //   await new Promise<void>(res => {
  //     // get witnesses for first level
  //     let counter = 0;
  //     async function startTask() {
  //       if (counter >= numberOfCalculations) {
  //         return;
  //       }

  //       let p = exec(
  //         `${witnessCalculatorPath} ${inputDir}/input${counter}.json ${witnessDir}/witness${counter}.wtns`,
  //       );
  //       p.then(() => {
  //         if (counter < numberOfCalculations) {
  //           promises.push(startTask());
  //           counter++;
  //         } else {
  //           res();
  //         }
  //       });
  //     }

  //     for (let i = 0; i < 8; i++) {
  //       promises.push(startTask());
  //       counter++;
  //     }
  //   });

  //   await Promise.all(promises);
  // }

  // await calculateWitness(
  //   '../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys',
  //   './inputs_first_level',
  //   './witnesses_first_level',
  //   SIZE / 64,
  // );

  // async function generateProofs(
  //   proovingKeyPath: string,
  //   witnessDir: string,
  //   proofDir: string,
  //   numberOfCalculations: number,
  // ) {
  //   await (async () => {
  //     for (let i = 0; i < numberOfCalculations; i++) {
  //       await exec(
  //         `../../../../vendor/rapidsnark/build/prover ${proovingKeyPath} ${witnessDir}/witness${i}.wtns ${proofDir}/proof${i}.json ${proofDir}/public${i}.json`,
  //       );
  //     }
  //   })();
  // }

  // await generateProofs(
  //   '../../build/aggregate_pubkeys/aggregate_pubkeys_0.zkey',
  //   './witnesses_first_level',
  //   './proofs_first_level',
  //   SIZE / 64,
  // );

  // let vkeyFirstLevel = JSON.parse(
  //   readFileSync('first-level-converted-vkey.json', 'utf-8'),
  // );

  // let vkeySecondLevel = JSON.parse(
  //   readFileSync('second-level-converted-vkey.json', 'utf-8'),
  // );

  // async function getRecursiveInput(
  //   offset: number,
  //   proofsDir: string,
  //   index: number,
  // ) {
  //   const epoch = Math.floor(beaconStateJson.slot / 32);
  //   const SIZE = 64;
  //   let points: PointG1[] = validators
  //     .slice(offset, offset + SIZE)
  //     .filter(
  //       curr =>
  //         curr.exitEpoch > epoch &&
  //         !curr.slashed &&
  //         curr.activationEpoch < epoch &&
  //         curr.activationEligibilityEpoch < epoch,
  //     )
  //     .map(x => PointG1.fromHex(x.pubkey));

  //   let sum = points.reduce((prev, curr) => prev.add(curr), PointG1.ZERO);

  //   const sumArr = bigint_to_array(55, 7, sum.toAffine()[0].value);
  //   sumArr.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

  //   const hashes: string[] = [];

  //   for (let i = 0; i < SIZE; i++) {
  //     hashes.push(
  //       bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[offset + i])),
  //     );
  //   }

  //   let n = 2;

  //   while (SIZE / n >= 1) {
  //     for (let i = 0; i < SIZE / n; i++) {
  //       hashes[i] = sha256(
  //         '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1]),
  //       );
  //     }

  //     n *= 2;
  //   }

  //   await exec(
  //     `python ${path.join(
  //       __dirname,
  //       '../../utils/proof_converter.py',
  //     )} ${proofsDir}/proof${index}.json ${proofsDir}/public${index}.json`,
  //   );

  //   const proof = JSON.parse(readFileSync(`proof.json`).toString());
  //   await rm('proof.json');

  //   return {
  //     currentEpoch: epoch,
  //     participantsCount: validators
  //       .slice(offset, offset + SIZE)
  //       .filter(
  //         curr =>
  //           curr.exitEpoch > epoch &&
  //           !curr.slashed &&
  //           curr.activationEpoch < epoch &&
  //           curr.activationEligibilityEpoch < epoch,
  //       ).length,
  //     hash: BigInt(hashes[0]).toString(2).padStart(256, '0').split(''),
  //     point: [
  //       bigint_to_array(55, 7, sum.toAffine()[0].value),
  //       bigint_to_array(55, 7, sum.toAffine()[1].value),
  //     ],
  //     ...proof,
  //   };
  // }

  // // first level
  // for (let i = 0; i < SIZE / 128; i++) {
  //   const first = await getRecursiveInput(i * 64, 'proofs_first_level', i);
  //   const second = await getRecursiveInput(
  //     i * 64 + 64,
  //     'proofs_first_level',
  //     i,
  //   );

  //   let input = {
  //     participantsCount: [first.participantsCount, second.participantsCount],
  //     currentEpoch: 160608,
  //     negpa: [first.negpa, second.negpa],
  //     pb: [first.pb, second.pb],
  //     pc: [first.pc, second.pc],
  //     hashes: [first.hash, second.hash],
  //     points: [first.point, second.point],
  //     negalfa1xbeta2: vkeyFirstLevel.negalfa1xbeta2,
  //     gamma2: vkeyFirstLevel.gamma2,
  //     delta2: vkeyFirstLevel.delta2,
  //     IC: vkeyFirstLevel.IC,
  //     prevNegalfa1xbeta2: [...Array(6).keys()].map(() =>
  //       [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
  //     ),
  //     prevGamma2: [...Array(2).keys()].map(() =>
  //       [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
  //     ),
  //     prevDelta2: [...Array(2).keys()].map(() =>
  //       [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
  //     ),
  //     prevIC: [...Array(2).keys()].map(() =>
  //       [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
  //     ),
  //     bitmask: [1, 1],
  //   };

  //   await writeFile(
  //     `inputs_second_level/input${i}.json`,
  //     JSON.stringify(input),
  //   );
  // }

  // await calculateWitness(
  //   '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_cpp/aggregate_pubkeys_verify',
  //   './inputs_second_level',
  //   './witnesses_second_level',
  //   SIZE / 128,
  // );

  // await generateProofs(
  //   '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_0.zkey',
  //   './witnesses_second_level',
  //   './proofs_second_level',
  //   SIZE / 128,
  // );
})();
