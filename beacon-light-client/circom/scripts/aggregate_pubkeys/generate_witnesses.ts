import { exec } from 'child_process';
import { promisify } from 'util';
const promiseExec = promisify(exec);

let counter = -1;

async function startTask() {
  let p = promiseExec(
    `../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys ./inputs/input${counter}.json ./witnesses/witness${counter}.wtns`,
  );
  p.then(() => {
    if (counter < 222) {
      counter++;
      startTask();
    }
  });
}
for (let i = 0; i < 32; i++) {
  counter++;
  startTask();
}
