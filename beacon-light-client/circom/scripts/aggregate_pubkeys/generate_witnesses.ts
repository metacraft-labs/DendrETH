import { exec } from 'child_process';
import { promisify } from 'util';
const promiseExec = promisify(exec);

let counter = 0;

async function startTask() {
  let p = promiseExec(
    `yarn snarkjs wej ./test_witness/witness${
      counter * 8 + 64
    }.wtns ./test_witness/witness${counter * 8 + 64}.json`,
  );

  // let p = promiseExec(
  //   `../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys ./test/input${
  //     counter * 8 + 64
  //   }.json ./test_witness/witness${counter * 8 + 64}.wtns`,
  // );
  p.then(() => {
    if (counter < 8) {
      startTask();
      counter++;
    }
  });
}
for (let i = 0; i < 8; i++) {
  startTask();
  counter++;
}
