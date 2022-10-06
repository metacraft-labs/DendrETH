import { PADDING } from './constants';
import { numerate, stringify } from './format';
import {
  extractFunctionID,
  extractFunctionSelector,
  getContractFunctions,
} from './function';
import { ArrayifiedContract, Function } from './types';

export const stringifyReport = (r: any, fids: any): string => {
  let rs = '';
  rs = rs.concat('BEACON LIGHT CLIENT\nGas Consumption Report\n\n');
  rs = rs.concat(
    'NOTE: Because of the way this script works there is currently a difference between the real gas cost and the measured one.\n',
  );
  rs = rs.concat(
    '      The problem can be fixed by walking through the script and reducing the estimated gas cost from the events with the corresponding difference (around ~1670 gas units)\n\n',
  );
  rs = rs.concat(
    '-----------------------------------------------------------------------------------\n\n',
  );

  for (let fSig of Object.keys(r)) {
    if (r[fSig].klgt === undefined) continue;
    const fId = extractFunctionID(extractFunctionSelector(fSig));
    const f: Function = fids[fId];
    rs = rs.concat(
      `${fSig}: ${formatNumberT(
        parseInt((r[fSig].gst / 2).toString()) /* temporary fix */,
        ',',
      )} units\n\n`,
    );
    for (let line of [/*...f.declaration, */ ...f.body]) {
      const ikl = Object.keys(r[fSig].klgt).includes(line.index.toString());
      !ikl
        ? (rs = rs.concat(line.content.slice(1)).concat('\n'))
        : (rs = rs
            .concat(line.content.slice(1))
            .concat(
              `\n\n    ^^^ ~${
                r[fSig].klgt[line.index] === 0
                  ? 'NaN'
                  : formatNumberT(
                      parseInt(
                        ((r[fSig].klgt[line.index] - 1670) / 2).toString(),
                      ) /* temporary fix */,
                      ',',
                    )
              } gas units spent ^^^\n\n`,
            ));
    }
    rs = rs.concat(
      '\n-----------------------------------------------------------------------------------\n\n',
    );
  }

  const s = rs.split('\n');
  return s
    .filter((_, i) => {
      return s[i].trim().length + s[i + 1]?.trim().length !== 0;
    })
    .join('\n');
};

export const findContractName = (contract: ArrayifiedContract) => {
  let name = '';
  for (let line of contract) {
    if (
      line.trim().startsWith('contract ') ||
      line.trim().startsWith('abstract contract ') ||
      line.trim().startsWith('library ')
    ) {
      name = line.trim().split(' ')[1];
    }
  }
  return name;
};

export const getContractsNames = (
  contracts: ArrayifiedContract[],
): { oldn: string; newn: string }[] => {
  const _contracts = [...contracts];
  const names: { oldn: string; newn: string }[] = [];
  for (let i = 0; i < _contracts.length; i++) {
    const oldn = findContractName([..._contracts[i]]);
    const newn = oldn + 'GasReportable';
    names[i] = { oldn, newn };
  }
  return names;
};

export const changeContractsNames = (
  contracts: ArrayifiedContract[],
  names: { oldn: string; newn: string }[],
) => {
  const _contracts = [...contracts];
  for (let i = 0; i < _contracts.length; i++) {
    for (let j = 0; j < names.length; j++) {
      _contracts[i] = stringify([..._contracts[i]])
        .replaceAll(new RegExp(`\\b${names[j].oldn}\\b`, 'g'), names[j].newn)
        .split('\n');
    }
  }
  return _contracts;
};

export const findKeyLineIdxs = (
  contract: ArrayifiedContract,
): { functionId: number; keyLineIdxs: Set<number> }[] => {
  const keyLineIdxs: Set<number> = new Set();
  const hasLibrary: boolean = contract.some(line =>
    line.trim().startsWith('library '),
  );

  let temp: number = -2;
  let b = true;
  let functions = hasLibrary
    ? getContractFunctions(
        numerate(contract).filter(line => {
          // ignore the libraries
          if (line.slice(PADDING).trim().startsWith('library ')) {
            temp = line.indexOf('library ');
            b = false;
            return false;
          } else if (line.indexOf('}') === temp) {
            b = true;
            temp = -2;
            return false;
          }
          return b;
        }),
      )
    : getContractFunctions(numerate(contract));

  for (let f of functions) {
    const linesInBlocksIdxs = f.blocks
      .map(block =>
        block.lines.map((line, i) => {
          i === 0 && keyLineIdxs.add(line.index - 1);
          i === block.lines.length - 1 && keyLineIdxs.add(line.index);
          return line.index;
        }),
      )
      .flat();
    for (let line of f.body) {
      if (!line.content.endsWith(';')) continue;
      if (linesInBlocksIdxs.includes(line.index)) continue;
      if (line.returns) continue;
      keyLineIdxs.add(line.index);
    }
  }

  // quick non-optimized cleaning
  for (let f of functions) {
    const linesInBlocksIdxs = f.blocks
      .map(block => block.lines.map(line => line.index))
      .flat();
    for (let i = 0; i < f.body.length; i++) {
      if (
        f.body[i].content.includes(' return ') ||
        f.body[i === 0 ? 0 : i - 1]?.content.includes(' return') ||
        linesInBlocksIdxs.includes(f.body[i].index)
      ) {
        keyLineIdxs.delete(f.body[i].index);
      }
    }
  }

  // add corresponding functions indexes to lines indexes
  const result: { functionId: number; keyLineIdxs: Set<number> }[] = [];
  for (let i = 0; i < functions.length; i++) {
    result[i] = {
      functionId: functions[i].id,
      keyLineIdxs: new Set(),
    };

    for (let keyLineIdx of keyLineIdxs.values())
      functions[i].body.some(line => line.index === keyLineIdx) &&
        result[i].keyLineIdxs.add(keyLineIdx);

    result[i].keyLineIdxs.add(
      functions[i].declaration[functions[i].declaration.length - 1].index,
    ); // this is the actual first line where a KeyLine event should emitted
  }

  return result;
};

export const markKeyLines = (contract: ArrayifiedContract) => {
  const _contract: ArrayifiedContract = [...contract];
  const r = findKeyLineIdxs(_contract);
  for (let _r of r) {
    for (let keyLineIdx of _r.keyLineIdxs.values()) {
      _contract[keyLineIdx] = _contract[keyLineIdx].concat(
        ` emit LogLine(${keyLineIdx}, ${_r.functionId}, gasleft());`,
      );
    }
  }
  return _contract;
};

export const formatNumberT = (n: number, y: string) => {
  let s = n.toString().split('').reverse().join('');
  let r = '';
  for (let i = 0; i < s.length; i++) {
    r = r.concat(s[i]);
    if (i % 3 == 2) r = r.concat(y);
  }

  //reverse
  r = r.split('').reverse().join('');

  // clean
  if (r.startsWith(y)) {
    r = r.slice(1);
  } else if (r.endsWith(y)) {
    r = r.slice(0, r.length - 1);
  }

  return r;
};
