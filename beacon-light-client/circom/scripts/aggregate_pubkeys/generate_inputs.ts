import * as validatorsJSON from '../../../../validators.json';
import { Fp, PointG1 } from '@noble/bls12-381';
import { bigint_to_array } from '../../../../libs/typescript/ts-utils/bls';
import { writeFile } from 'fs';
import { exec } from 'child_process';
import { promisify } from 'util';
const promiseExec = promisify(exec);

const validators = validatorsJSON as any;
let writes = [];
// for(let i = 0; i <= validators.data.length / 2048; i++) {
//   let points: PointG1[] = (validators as any).data.slice(i * 2048, (i * 2048) + 2048).map(x =>
//     PointG1.fromHex(x.validator.pubkey.slice(2)),
//   );

//   if(points.length < 2048) {
//     break;
//   }

//   let input = {
//     points: [
//       points.map(x => [
//         bigint_to_array(55, 7, x.toAffine()[0].value),
//         bigint_to_array(55, 7, x.toAffine()[1].value),
//       ]),
//     ]
//   };

//   writeFile(`./witnesses/input${i}.json`, JSON.stringify(input), ( )=> {});
// }

// let counter = 0;

async function startTask(start, end) {
  let p = promiseExec(
    `yarn ts-node --transpile-only  ./do_smart_stuff.ts ${start} ${end}`,
  );
}
let totalFiles = validators.data.length / 2048;
let numberOfChunks = 32;
let chunkSize = Math.floor(totalFiles / numberOfChunks);
for (let i = 0; i < numberOfChunks; i++) {
  startTask(i * chunkSize, (i + 1) * chunkSize);
}
startTask(numberOfChunks * chunkSize, totalFiles);

// let counter = 0;

// async function startTask() {

//   let p = promiseExec(`../../build/aggregate_pubkeys/aggregate_pubkeys_cpp/aggregate_pubkeys ./witnesses/input${counter}.json witness${counter}.wtns`);
//   p.then(() => {
//     if (counter < 2048) {
//       counter++;
//       startTask();
//     }
//   })

// }
// for(let i = 0; i < 32; i++) {
//   startTask();
// }
