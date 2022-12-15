import { ssz } from '@chainsafe/lodestar-types';
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
  let n = 2;
  let level = 1;
  while (len / n >= 2) {
    for (let i = 0; i < len / n; i++) {
      pairs[i] = getRecursiveInput(
        pairs[i].point.add(pairs[i + 1].point),
        sha256(formatHex(pairs[i].hash) + formatHex(pairs[i].hash)),
        pairs[i].participantsCount + pairs[i + 1].participantsCount,
        './proofs_second_level',
        i
      );
    }

    if ((len / n) % 2 != 0) {
      pairs[len / n + 1] = JSON.parse(
        readFileSync(`zeros/input${level}.json`, 'utf-8')
      );
    }

    await getProofsFromPairs(
      pairs.slice(0, len / n),
      vkeySecondLevel,
      calculateWitness,
      generateProofs
    );

    n *= 2;
    level++;
  }

  // fill in the zero-levels
  while (level < 34) {
    pairs[1] = JSON.parse(readFileSync(`zeros/input${level}.json`, 'utf-8'));

    await getProofsFromPairs(
      pairs.slice(0, 2),
      vkeySecondLevel,
      calculateWitness,
      generateProofs
    );

    level++;
  }
}

async function getSecondLevelProofs() {
  let pairs: any[] = [];
  for (let i = 0; i < SIZE / 64; i++) {
    let points: PointG1[] = validators
      .slice(i * 64, i * 64 + 64)
      .filter(
        curr => curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch
      )
      .map(x => PointG1.fromHex(x.pubkey));

    const participantsCount = validators
      .slice(i * 64, i * 64 + 64)
      .filter(
        curr => curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch
      ).length;

    const hashes: string[] = [];

    for (let i = 0; i < 64; i++) {
      hashes.push(
        bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[i * 64 + i]))
      );
    }

    let n = 2;

    while (64 / n >= 1) {
      for (let i = 0; i < 64 / n; i++) {
        hashes[i] = sha256(
          '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1])
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
        i
      )
    );
  }

  if (SIZE % 64 != 0) {
    let points: PointG1[] = validators
      .slice((SIZE / 64) * 64, (SIZE / 64) * 64 + (SIZE % 64))
      .filter(
        curr => curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch
      )
      .map(x => PointG1.fromHex(x.pubkey));

    const participantsCount = validators
      .slice((SIZE / 64) * 64, (SIZE / 64) * 64 + (SIZE % 64))
      .filter(
        curr => curr.exitEpoch > epoch &&
          !curr.slashed &&
          curr.activationEpoch < epoch &&
          curr.activationEligibilityEpoch < epoch
      ).length;

    const hashes: string[] = [];

    for (let i = 0; i < points.length; i++) {
      hashes.push(
        bytesToHex(ssz.phase0.Validator.hashTreeRoot(validators[i * 64 + i]))
      );
    }

    for (let i = points.length; i < 64; i++) {
      hashes.push(''.padEnd(64, '0'));
    }

    let n = 2;

    while (64 / n >= 1) {
      for (let i = 0; i < 64 / n; i++) {
        hashes[i] = sha256(
          '0x' + formatHex(hashes[2 * i]) + formatHex(hashes[2 * i + 1])
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
        SIZE / 64
      )
    );
  }

  if (pairs.length % 2 != 0) {
    pairs.push(JSON.parse(readFileSync('zeros/input0.json', 'utf-8')));
  }

  await getProofsFromPairs(
    pairs,
    vkeyFirstLevel,
    calculateWitness,
    generateProofs
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
      validators.slice((SIZE / 64) * 64, (SIZE / 64) * 64 + (SIZE % 64)),
      SIZE / 64,
      Math.floor(beaconStateJson.slot / 32),
    );
  }

  // calculate the witnesses for the first level
  await calculateWitness(
    '../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys',
    './inputs_first_level',
    './witnesses_first_level',
    SIZE / 64,
  );

  // generate the proofs for the first level
  await generateProofs(
    '../../build/aggregate_pubkeys/aggregate_pubkeys_0.zkey',
    './witnesses_first_level',
    './proofs_first_level',
    SIZE / 64,
  );
}

async function getProofsFromPairs(
  pairs: any[],
  vkey: any,
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
      prevNegalfa1xbeta2: [...Array(6).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      prevGamma2: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      prevDelta2: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      prevIC: [...Array(2).keys()].map(() =>
        [...Array(2).keys()].map(() => [...Array(6).keys()].map(() => 0)),
      ),
      bitmask: [first.bitmask, second.bitmask],
    };

    await writeFile(
      `inputs_second_level/input${i}.json`,
      JSON.stringify(input),
    );
  }

  await calculateWitness(
    '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_cpp/aggregate_pubkeys_verify',
    './inputs_second_level',
    './witnesses_second_level',
    SIZE / 128,
  );

  await generateProofs(
    '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_0.zkey',
    './witnesses_second_level',
    './proofs_second_level',
    SIZE / 128,
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
  await (async () => {
    for (let i = 0; i < numberOfCalculations; i++) {
      await exec(
        `../../../../vendor/rapidsnark/build/prover ${proovingKeyPath} ${witnessDir}/witness${i}.wtns ${proofDir}/proof${i}.json ${proofDir}/public${i}.json`,
      );
    }
  })();
}

async function getRecursiveInput(
  sum: PointG1,
  hash: string,
  participantsCount: number,
  proofsDir: string,
  index: number,
) {
  const sumArr = bigint_to_array(55, 7, sum.toAffine()[0].value);
  sumArr.push(...bigint_to_array(55, 7, sum.toAffine()[1].value));

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
