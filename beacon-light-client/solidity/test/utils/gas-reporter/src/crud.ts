import * as fs from 'fs';

import { ArrayifiedContract, StringifiedContract } from './types';
import { CONTRACTS_ORIG_PATH, CONTRACTS_TEMP_PATH } from './constants';
import { getFilesInDir } from '../..';
import { arrayify } from './format';

export const rewriteImports = (c: ArrayifiedContract): ArrayifiedContract => {
  for (let i = 0; i < c.length; i++) {
    if (c[i].trim().startsWith('import ')) {
      if (c[i].includes('console.sol')) continue;
      const p = c[i].includes('"') ? '"' : "'";
      const h = c[i].slice(c[i].indexOf(p) + 1, c[i].lastIndexOf('/'));
      if (h.includes('@') || h.includes('node_modules')) continue;
      const n = '.';
      c[i] = c[i].replace(h, n);
    }
  }
  return c;
};

export const getContracts = (): ArrayifiedContract[] => {
  return getFilesInDir(CONTRACTS_ORIG_PATH).map(arrayify);
};

export const writeContract = (
  c: StringifiedContract,
  d: string,
): StringifiedContract => {
  fs.writeFileSync(d, c);
  return c;
};

export const clearTempDir = () => {
  fs.rmSync(CONTRACTS_TEMP_PATH, {
    recursive: true,
    force: true,
  });
};
