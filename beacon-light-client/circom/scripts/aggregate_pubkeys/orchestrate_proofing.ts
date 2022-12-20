import { ssz } from '@chainsafe/lodestar-types';
import { PointG1, Fp } from '@noble/bls12-381';
import { exec as _exec } from 'child_process';
import { readFileSync, rmSync } from 'fs';
import { promisify } from 'util';
import {
  array_to_bigint,
  bigint_to_array,
  bytesToHex,
  formatHex,
} from '../../../../libs/typescript/ts-utils/bls';
import getInput from './get-aggregate-pubkeys-input';
import { sha256 } from 'ethers/lib/utils';
import path from 'path';
import { rm, writeFile } from 'fs/promises';

const exec = promisify(_exec);

const SIZE = 191;

let validatorsJSON = JSON.parse(
  readFileSync('../../../../validators.json', 'utf-8'),
);

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

let beaconStateJson = JSON.parse(
  readFileSync('../../../../beacon_state.json', 'utf-8'),
).data;

const epoch = Math.floor(beaconStateJson.slot / 32);

let vkeyFirstLevel = JSON.parse(
  readFileSync('first-level-converted-vkey.json', 'utf-8'),
);

let vkeySecondLevel = JSON.parse(
  readFileSync('second-level-converted-vkey.json', 'utf-8'),
);

(async () => {
  // get inputs for validators
  await getFirstLevelProofs();

  let pairs: any[] = await getSecondLevelProofs();

  await getRecursiveProofs(pairs);
})();

async function getRecursiveProofs(pairs: any[]) {
  let len = pairs.length;
  let level = 1;
  while (len >= 2) {
    let index = 0;
    for (let i = 0; i < len; i += 2) {
      let point =
        pairs[i].bitmask == 1
          ? new PointG1(
              new Fp(array_to_bigint(55, pairs[i].point[0])),
              new Fp(array_to_bigint(55, pairs[i].point[1])),
            )
          : PointG1.ZERO;

      let point2 =
        pairs[i + 1].bitmask == 1
          ? new PointG1(
              new Fp(array_to_bigint(55, pairs[i + 1].point[0])),
              new Fp(array_to_bigint(55, pairs[i + 1].point[1])),
            )
          : PointG1.ZERO;

      pairs[index] = await getRecursiveInput(
        point.add(point2),
        sha256(
          '0x' +
            formatHex(
              BigInt('0b' + pairs[i].hash.join(''))
                .toString(16)
                .padStart(64, '0'),
            ) +
            formatHex(
              BigInt('0b' + pairs[i + 1].hash.join(''))
                .toString(16)
                .padStart(64, '0'),
            ),
        ),
        pairs[i].participantsCount + pairs[i + 1].participantsCount,
        './proofs_second_level',
        index,
      );
      index++;
    }

    if (index % 2 != 0) {
      pairs[index] = JSON.parse(
        readFileSync(`zeros_input/input${level}.json`, 'utf-8'),
      );
      index++;
    }

    // console.log(JSON.stringify(pairs));
    // console.log(index);

    await getProofsFromPairs(
      pairs.slice(0, index),
      vkeySecondLevel,
      level == 1 ? vkeyFirstLevel : vkeySecondLevel,
      calculateWitness,
      generateProofs,
    );

    len /= 2;
    level++;
  }

  // fill in the zero-levels
  while (level < 34) {
    console.log('here');
    console.log(level);

    pairs[0] = await getRecursiveInput(
      (pairs[0].bitmask == 1
        ? new PointG1(
            new Fp(array_to_bigint(55, pairs[0].point[0])),
            new Fp(array_to_bigint(55, pairs[0].point[1])),
          )
        : PointG1.ZERO
      ).add(
        pairs[1].bitmask == 1
          ? new PointG1(
              new Fp(array_to_bigint(55, pairs[1].point[0])),
              new Fp(array_to_bigint(55, pairs[1].point[1])),
            )
          : PointG1.ZERO,
      ),
      sha256(
        '0x' +
          formatHex(
            BigInt('0b' + pairs[0].hash.join(''))
              .toString(16)
              .padStart(64, '0'),
          ) +
          formatHex(
            BigInt('0b' + pairs[1].hash.join(''))
              .toString(16)
              .padStart(64, '0'),
          ),
      ),
      pairs[0].participantsCount + pairs[1].participantsCount,
      './proofs_second_level',
      0,
    );

    pairs[1] = JSON.parse(
      readFileSync(`zeros_input/input${level}.json`, 'utf-8'),
    );

    await getProofsFromPairs(
      pairs.slice(0, 2),
      vkeySecondLevel,
      level == 1 ? vkeyFirstLevel : vkeySecondLevel,
      calculateWitness,
      generateProofs,
    );

    level++;
  }
}

async function getSecondLevelProofs() {
  let pairs: any[] = [];
  for (let i = 0; i < Math.floor(SIZE / 64); i++) {
    let points: PointG1[] = validators
      .slice(i * 64, i * 64 + 64)
      .filter(
        curr =>
          curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch,
      )
      .map(x => PointG1.fromHex(x.pubkey));

    const participantsCount = validators
      .slice(i * 64, i * 64 + 64)
      .filter(
        curr =>
          curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch,
      ).length;

    const hashes: string[] = [];

    for (let j = 0; j < 64; j++) {
      hashes.push(
        bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[i * 64 + j])),
      );
    }

    let n = 2;

    while (64 / n >= 1) {
      for (let i = 0; i < 64 / n; i++) {
        hashes[i] = sha256(
          '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1]),
        );
      }

      n *= 2;
    }

    pairs.push(
      await getRecursiveInput(
        points.reduce((sum, curr) => sum.add(curr), PointG1.ZERO),
        hashes[0],
        participantsCount,
        'proofs_first_level',
        i,
      ),
    );
  }

  if (SIZE % 64 != 0) {
    let points: PointG1[] = validators
      .slice(
        Math.floor(SIZE / 64) * 64,
        Math.floor(SIZE / 64) * 64 + (SIZE % 64),
      )
      .filter(
        curr =>
          curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch,
      )
      .map(x => PointG1.fromHex(x.pubkey));

    const participantsCount = validators
      .slice(
        Math.floor(SIZE / 64) * 64,
        Math.floor(SIZE / 64) * 64 + (SIZE % 64),
      )
      .filter(
        curr =>
          curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch,
      ).length;

    const hashes: string[] = [];

    for (let i = 0; i < points.length; i++) {
      hashes.push(
        bytesToHex(
          ssz.phase0.Validator.hashTreeRoot(
            validators[Math.floor(SIZE / 64) * 64 + i],
          ),
        ),
      );
    }

    for (let i = points.length; i < 64; i++) {
      hashes.push(''.padEnd(64, '0'));
    }

    let n = 2;

    while (64 / n >= 1) {
      for (let i = 0; i < 64 / n; i++) {
        hashes[i] = sha256(
          '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1]),
        );
      }

      n *= 2;
    }

    pairs.push(
      await getRecursiveInput(
        points.reduce((sum, curr) => sum.add(curr), PointG1.ZERO),
        hashes[0],
        participantsCount,
        'proofs_first_level',
        Math.floor(SIZE / 64),
      ),
    );
  }

  if (pairs.length % 2 != 0) {
    pairs.push(JSON.parse(readFileSync('zeros_input/input0.json', 'utf-8')));
  }

  // console.log(JSON.stringify(pairs));

  await getProofsFromPairs(
    pairs,
    vkeyFirstLevel,
    {
      negalfa1xbeta2: [...Array(6).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      gamma2: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      delta2: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      IC: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
    },
    calculateWitness,
    generateProofs,
  );
  return pairs;
}

async function getFirstLevelProofs() {
  for (let i = 0; i < SIZE / 64; i++) {
    await getInput(
      validators.slice(i * 64, i * 64 + 64),
      i,
      Math.floor(beaconStateJson.slot / 32),
    );
  }

  // get incomplete input
  if (SIZE % 64 != 0) {
    await getInput(
      validators.slice(
        Math.floor(SIZE / 64) * 64,
        Math.floor(SIZE / 64) * 64 + (SIZE % 64),
      ),
      Math.floor(SIZE / 64),
      Math.floor(beaconStateJson.slot / 32),
    );
  }

  // calculate the witnesses for the first level
  await calculateWitness(
    '../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys',
    './inputs_first_level',
    './witnesses_first_level',
    SIZE % 64 == 0 ? Math.floor(SIZE / 64) : Math.floor(SIZE / 64) + 1,
  );

  // generate the proofs for the first level
  await generateProofs(
    '../../build/aggregate_pubkeys/aggregate_pubkeys_0.zkey',
    './witnesses_first_level',
    './proofs_first_level',
    SIZE % 64 == 0 ? Math.floor(SIZE / 64) : Math.floor(SIZE / 64) + 1,
  );
}

async function getProofsFromPairs(
  pairs: any[],
  vkey: any,
  prevVkey: any,
  calculateWitness: (
    witnessCalculatorPath: string,
    inputDir: string,
    witnessDir: string,
    numberOfCalculations: number,
  ) => Promise<void>,
  generateProofs: (
    proovingKeyPath: string,
    witnessDir: string,
    proofDir: string,
    numberOfCalculations: number,
  ) => Promise<void>,
) {
  // console.log(pairs.length);
  // console.log(JSON.stringify(pairs));
  for (let i = 0; i < pairs.length; i += 2) {
    const first = pairs[i];
    const second = pairs[i + 1];

    let input = {
      participantsCount: [first.participantsCount, second.participantsCount],
      currentEpoch: 160608,
      negpa: [first.negpa, second.negpa],
      pb: [first.pb, second.pb],
      pc: [first.pc, second.pc],
      hashes: [first.hash, second.hash],
      points: [first.point, second.point],
      negalfa1xbeta2: vkey.negalfa1xbeta2,
      gamma2: vkey.gamma2,
      delta2: vkey.delta2,
      IC: vkey.IC,
      prevNegalfa1xbeta2: prevVkey.negalfa1xbeta2,
      prevGamma2: prevVkey.gamma2,
      prevDelta2: prevVkey.delta2,
      prevIC: prevVkey.IC,
      bitmask: [first.bitmask, second.bitmask],
    };

    await writeFile(
      `inputs_second_level/input${i / 2}.json`,
      JSON.stringify(input),
    );
  }

  await calculateWitness(
    '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_cpp/aggregate_pubkeys_verify',
    './inputs_second_level',
    './witnesses_second_level',
    Math.floor(pairs.length / 2),
  );

  await generateProofs(
    '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_0.zkey',
    './witnesses_second_level',
    './proofs_second_level',
    Math.floor(pairs.length / 2),
  );
}

async function calculateWitness(
  witnessCalculatorPath: string,
  inputDir: string,
  witnessDir: string,
  numberOfCalculations: number,
) {
  let promises: Promise<void>[] = [];
  await new Promise<void>(res => {
    // get witnesses for first level
    let counter = 0;
    async function startTask() {
      if (counter >= numberOfCalculations) {
        return;
      }

      let p = exec(
        `${witnessCalculatorPath} ${inputDir}/input${counter}.json ${witnessDir}/witness${counter}.wtns`,
      );
      p.then(() => {
        if (counter < numberOfCalculations) {
          promises.push(startTask());
          counter++;
        } else {
          res();
        }
      });
    }

    for (let i = 0; i < 8; i++) {
      promises.push(startTask());
      counter++;
    }
  });

  await Promise.all(promises);
}

async function generateProofs(
  proovingKeyPath: string,
  witnessDir: string,
  proofDir: string,
  numberOfCalculations: number,
) {
  for (let i = 0; i < numberOfCalculations; i++) {
    await exec(
      `../../../../vendor/rapidsnark/build/prover ${proovingKeyPath} ${witnessDir}/witness${i}.wtns ${proofDir}/proof${i}.json ${proofDir}/public${i}.json`,
    );
  }
}

async function getRecursiveInput(
  sum: PointG1,
  hash: string,
  participantsCount: number,
  proofsDir: string,
  index: number,
) {
  // console.log('-----------------');
  // console.log(sum);
  // console.log(hash);
  // console.log(participantsCount);
  // console.log(proofsDir);
  // console.log(index);
  // console.log('-----------------');
  // const sumArr = bigint_to_array(55, 7, sum.toAffine()[0].value);
  // sumArr.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

  await exec(
    `python ${path.join(
      __dirname,
      '../../utils/proof_converter.py',
    )} ${proofsDir}/proof${index}.json ${proofsDir}/public${index}.json`,
  );

  const proof = JSON.parse(readFileSync(`proof.json`).toString());
  await rm('proof.json');

  return {
    currentEpoch: epoch,
    participantsCount: participantsCount,
    hash: BigInt(hash).toString(2).padStart(256, '0').split(''),
    point: [
      bigint_to_array(55, 7, sum.toAffine()[0].value),
      bigint_to_array(55, 7, sum.toAffine()[1].value),
    ],
    bitmask: 1,
    ...proof,
  };
}
