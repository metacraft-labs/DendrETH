import { ethers } from 'hardhat';
import { ArrayifiedContract } from './types';

export const setupGasReportable = (c: ArrayifiedContract) => {
  for (let i = 0; i < c.length; i++) {
    if (!c[i].includes('pragma solidity ')) continue;
    c[i + 1] = c[i + 1] + 'import "./GasReportable.sol";';
    break;
  }

  for (let i = 0; i < c.length; i++) {
    if (
      !c[i].trim().startsWith('contract ') &&
      !c[i].trim().startsWith('abstract contract ')
    )
      continue;
    const s =
      c[i].indexOf('contract ') === -1
        ? c[i].indexOf('abstract contract ')
        : c[i].indexOf('contract ');

    if (c[i].includes(' is ')) {
      c[i] =
        c[i].slice(0, c[i].indexOf(' is ') + ' is '.length) +
        ' GasReportable, ' +
        c[i].slice(c[i].indexOf(' is ') + ' is '.length);
    } else {
      c[i] =
        c[i].slice(0, c[i].indexOf('{')) +
        ' is GasReportable ' +
        c[i].slice(c[i].indexOf('{'));
    }

    for (let j = i + 1; j < c.length; j++) {
      if (c[j].indexOf('}') === s) break;
      if (!c[j].trim().startsWith('function ')) continue;

      let f = '';
      for (let z = j; z < c.length; z++) {
        f = f.concat(c[z].trim());
        if (!c[z].includes(')')) continue;
        f = f.slice(
          f.indexOf('function ') + 'function '.length,
          f.indexOf(')') + 1,
        );
        const n = f.slice(0, f.indexOf('(')).trim();
        const p = f.slice(f.indexOf('(') + 1, f.indexOf(')')).trim();

        let sig: string;
        if (p === '') {
          sig = n + '()';
        } else {
          sig =
            n +
            '(' +
            p
              .trim()
              .split(',')
              .map(x => x.trim().split(' ')[0])
              .join(',') +
            ')';
        }
        const sel = ethers.utils
          .keccak256(ethers.utils.toUtf8Bytes(sig))
          .slice(0, 10);

        c[z] =
          c[z].slice(0, c[z].indexOf(')') + 1) +
          ` gas_report(${Number(sel)}) ` +
          c[z].slice(c[z].indexOf(')') + 1);
        j = z;
        break;
      }
    }
  }

  return c;
};
