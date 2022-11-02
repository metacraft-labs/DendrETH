import { exec } from 'child_process';
import { promisify } from 'util';
const promiseExec = promisify(exec);

(async () => {
  for (let i = 0; i < 222; i++) {
    await promiseExec(
      `../../../../vendor/rapidsnark/build/prover ../../build/aggregate_pubkeys/aggregate_pubkeys_0.zkey witnesses/witness${i}.wtns proofs/proof${i}.json proofs/public${i}.json`,
    );
  }
})();
