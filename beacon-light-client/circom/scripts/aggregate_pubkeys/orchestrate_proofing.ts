import { ssz } from '@chainsafe/lodestar-types';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { ByteVectorType, UintNumberType, BooleanType } from '@chainsafe/ssz';
import { ValueOfFields } from '@chainsafe/ssz/lib/view/container';
import { PointG1 } from '@noble/bls12-381';
import { rejects } from 'assert';
import { exec as _exec } from 'child_process';
import { count } from 'console';
import { readFileSync, writeFileSync } from 'fs';
import { writeFile } from 'fs/promises';
import { resolve } from 'path';
import { promisify } from 'util';
import { Worker } from 'worker_threads';
import {
  bigint_to_array,
  bytesToHex,
} from '../../../../libs/typescript/ts-utils/bls';

const exec = promisify(_exec);

const SIZE = 32768;

let validatorsJSON = JSON.parse(
  readFileSync('../../../../validators.json', 'utf-8'),
);

let validators = ssz.phase0.Validators.fromJson(
  (validatorsJSON as any).data.slice(0, SIZE).map(x => x.validator),
);

let beaconStateJson = JSON.parse(
  readFileSync('../../../../beacon_state.json', 'utf-8'),
).data;

let workers: Worker[] = [];

for (var i = 0; i < 32; i++) {
  workers.push(
    new Worker('./get-aggregate-pubkeys-input.ts', {
      workerData: {
        index: i,
      },
    }),
  );
}

// get inputs
let counter = 0;
let finished = 0;
let inputsPromise = new Promise<void>((resolve, reject) => {
  for (let i = 0; i < 32; i++) {
    workers[i].postMessage({
      validators: validators.slice(counter * 64, counter * 64 + 64),
      index: counter++,
      epoch: Math.floor(beaconStateJson.slot / 32),
    });

    workers[i].on('message', index => {
      if (counter < SIZE / 64) {
        workers[index].postMessage({
          validators: validators.slice(counter * 64, counter * 64 + 64),
          index: counter++,
          epoch: Math.floor(beaconStateJson.slot / 32),
        });
      } else {
        finished++;
        if (finished == 32) {
          resolve();
        }
        workers[index].terminate();
      }
    });
  }
});
inputsPromise.then(() => {
  console.log('now');

  let witnessPromise = new Promise<void>((res, rej) => {
    // get witnesses for first level
    counter = 0;
    async function startTask() {
      let p = exec(
        `../../build/aggregate_pubkeys/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys ./inputs_first_level/input${counter}.json ./witnesses_first_level/witness${counter}.wtns`,
      );
      p.then(() => {
        if (counter < SIZE / 64 - 1) {
          counter++;
          startTask();
        } else {
          res();
        }
      });
    }

    for (let i = 0; i < 8; i++) {
      counter++;
      startTask();
    }
  });

  witnessPromise.then(() => {
    (async () => {
      for (let i = 0; i < SIZE / 64; i++) {
        await exec(
          `../../../../vendor/rapidsnark/build/prover ../../build/aggregate_pubkeys/aggregate_pubkeys/aggregate_pubkeys_0.zkey witnesses_first_level/witness${i}.wtns proofs/proof${i}.json proofs/public${i}.json`,
        );
      }
    })();
  });
});
