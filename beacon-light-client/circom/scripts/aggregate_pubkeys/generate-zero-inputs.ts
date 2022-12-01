import { exec as _exec } from 'child_process';
import { readFile, readFileSync } from 'fs';
import { writeFile, rm } from 'fs/promises';
import path from 'path';
import { promisify } from 'util';
import { formatHex } from '../../../../libs/typescript/ts-utils/bls';
import { sha256 } from 'ethers/lib/utils';

const exec = promisify(_exec);

(async () => {
  let input = {
    points: [...Array(64).keys()].map(() => [
      [
        '16589478066046651',
        '22658679592837110',
        '35004527604248919',
        '16789302793630161',
        '7530538873701625',
        '32234187716135413',
        '1684953952445941',
      ],
      [
        '11860609209853921',
        '4091579406338073',
        '12578493816062195',
        '13088963032438982',
        '24961455755233629',
        '8501508834176643',
        '612415636564648',
      ],
    ]),
    zero: [...[...Array(64).keys()].map(() => 0)],
    withdrawCredentials: [...Array(64).keys()].map(() =>
      ''.padStart(256, '0').split(''),
    ),
    effectiveBalance: [...Array(64).keys()].map(() =>
      ''.padStart(256, '0').split(''),
    ),
    slashed: [...Array(64).keys()].map(() => '0'),
    activationEligibilityEpoch: [...Array(64).keys()].map(() => '0'),
    activationEpoch: [...Array(64).keys()].map(() => '0'),
    exitEpoch: [...Array(64).keys()].map(() => '0'),
    withdrawableEpoch: [...Array(64).keys()].map(() =>
      ''.padStart(256, '0').split(''),
    ),
    bitmask: [...Array(64).keys()].map(() => 0),
    currentEpoch: 160608,
  };

  await writeFile(`input.json`, JSON.stringify(input));

  await exec(
    '../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys input.json witness.wtns',
  );

  await exec(
    '../../../../vendor/rapidsnark/build/prover ../../build/aggregate_pubkeys/aggregate_pubkeys_0.zkey witness.wtns proof.json public.json',
  );

  await exec(
    `python ${path.join(
      __dirname,
      '../../utils/proof_converter.py',
    )} proof.json public.json`,
  );

  let zeros: string[] = [];
  zeros[0] = ''.padStart(64, '0');

  for (let i = 1; i <= 6; i++) {
    zeros[i] = formatHex(sha256('0x' + zeros[i - 1] + zeros[i - 1]));
  }

  const proof = JSON.parse(readFileSync(`proof.json`).toString());

  await writeFile(
    `zeros_input/input0.json`,
    JSON.stringify({
      currentEpoch: 160608,
      participantsCount: 0,
      hash: BigInt('0x' + zeros[6])
        .toString(2)
        .padStart(256, '0')
        .split(''),
      point: input.points[0],
      bitmask: 0,
      ...proof,
      pubInput: undefined,
    }),
  );

  const zeroVk = {
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
  };

  let firstLevelVk = JSON.parse(
    readFileSync('first-level-converted-vkey.json', 'utf-8'),
  );

  let secondLevelVk = JSON.parse(
    readFileSync('second-level-converted-vkey.json', 'utf-8'),
  );

  for (let i = 7; i < 40; i++) {
    const prevInput = JSON.parse(
      readFileSync(`zeros_input/input${i - 7}.json`, 'utf-8'),
    );

    let input = {
      participantsCount: [0, 0],
      currentEpoch: prevInput.currentEpoch,
      negpa: [prevInput.negpa, prevInput.negpa],
      pb: [prevInput.pb, prevInput.pb],
      pc: [prevInput.pc, prevInput.pc],
      hashes: [prevInput.hash, prevInput.hash],
      points: [prevInput.point, prevInput.point],
      negalfa1xbeta2:
        i == 7 ? firstLevelVk.negalfa1xbeta2 : secondLevelVk.negalfa1xbeta2,
      gamma2: i == 7 ? firstLevelVk.gamma2 : secondLevelVk.gamma2,
      delta2: i == 7 ? firstLevelVk.delta2 : secondLevelVk.delta2,
      IC: i == 7 ? firstLevelVk.IC : secondLevelVk.IC,
      prevNegalfa1xbeta2:
        i == 7
          ? zeroVk.negalfa1xbeta2
          : i == 8
          ? firstLevelVk.negalfa1xbeta2
          : secondLevelVk.negalfa1xbeta2,
      prevGamma2:
        i == 7
          ? zeroVk.gamma2
          : i == 8
          ? firstLevelVk.gamma2
          : secondLevelVk.gamma2,
      prevDelta2:
        i == 7
          ? zeroVk.delta2
          : i == 8
          ? firstLevelVk.delta2
          : secondLevelVk.delta2,
      prevIC: i == 7 ? zeroVk.IC : i == 8 ? firstLevelVk.IC : secondLevelVk.IC,
      bitmask: [0, 0],
    };

    await writeFile(`input.json`, JSON.stringify(input));

    await exec(
      '../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_cpp/aggregate_pubkeys_verify input.json witness.wtns',
    );

    await exec(
      '../../../../vendor/rapidsnark/build/prover ../../build/aggregate_pubkeys_verify/aggregate_pubkeys_verify_0.zkey witness.wtns proof.json public.json',
    );

    await exec(
      `python ${path.join(
        __dirname,
        '../../utils/proof_converter.py',
      )} proof.json public.json`,
    );

    const proof = JSON.parse(readFileSync(`proof.json`).toString());

    zeros[i] = formatHex(sha256('0x' + zeros[i - 1] + zeros[i - 1]));

    await writeFile(
      `zeros_input/input${i - 6}.json`,
      JSON.stringify({
        currentEpoch: 160608,
        participantsCount: 0,
        hash: BigInt('0x' + zeros[i])
          .toString(2)
          .padStart(256, '0')
          .split(''),
        point: input.points[0],
        bitmask: 0,
        ...proof,
        pubInput: undefined,
      }),
    );
  }

  await rm('input.json');
  await rm('witness.wtns');
  await rm('proof.json');
  await rm('public.json');
})();
