import { buildPoseidon } from 'circomlibjs';
import { bytesToHex } from '@dendreth/utils/ts-utils/bls';

(async () => {
  const poseidon = await buildPoseidon();

  const res = poseidon([1, 2], 0, 4);

  console.log(res);
})();
