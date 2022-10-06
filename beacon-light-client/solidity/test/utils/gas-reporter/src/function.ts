import { ethers } from 'hardhat';
import { BLOCK_TYPES, PADDING, SOLIDITY_STORAGE_LOCATIONS } from './constants';
import {
  ArrayifiedContract,
  Block,
  Body,
  Declaration,
  Event,
  Function,
  Line,
  Variable,
} from './types';

// ================= //
// |  MAIN EXPORT  | //
// ================= //

// pass only `numerated` arrayified contracts
export const getContractFunctions = (contract: ArrayifiedContract) => {
  return extractFunctions(contract).map(formatFunction);
};

// ================ //
// |   ABSTRACT   | //
// ================ //

const formatFunction = (func: string[]): Function => {
  const result = {} as Function;

  result.name = extractFunctionName(func);
  result.body = extractFunctionBody(func);
  result.declaration = extractFunctionDeclaration(func);

  result.firstLine = extractFunctionFirstLine(result.body);
  result.lastLines = extractFunctionLastLines(result.body);
  result.events = extractFunctionEvents(result.body);
  result.blocks = extractFunctionBlocks(result.body);

  result.params = extractFunctionParams(result.declaration);
  result.modifiers = extractFunctionModifiers(result.declaration);
  result.returns = extractFunctionReturns(result.declaration);

  result.signature = extractFunctionSignature(func);
  result.selector = extractFunctionSelector(result.signature);
  result.id = extractFunctionID(result.selector);

  return result;
};

// pass only `numerated` arrayified contracts
const extractFunctions = (contract: ArrayifiedContract): string[][] => {
  const functions: string[][] = [];
  for (let i = 0; i < contract.length; i++) {
    if (contract[i].slice(PADDING).trim().startsWith('function ')) {
      const start = contract[i].indexOf('function ');
      const func: string[] = [];
      for (let j = i; j < contract.length; j++) {
        func.push(contract[j]);
        if (contract[j].indexOf('}') === start) {
          functions.push(func);
          i = j;
          break;
        }
      }
    }
  }
  return functions;
};

// ============= //
// |   LOGIC   | //
// ============= //

const extractFunctionName = (func: string[]): string => {
  const line = func.find(f => f.slice(PADDING).trim().startsWith('function '));
  if (line === undefined)
    throw new Error('Function.extractName: Invalid function passed!');

  const i = line.indexOf('function ') + 'function '.length;
  const j = line.indexOf('(');

  return line.slice(i, j);
};

const extractFunctionBody = (func: string[]): Body => {
  const joined = func.slice(0, func.length - 1).join('\n');
  const body = joined
    .slice(joined.indexOf('{') + 1)
    .trim()
    .split('\n');
  return body.map(formatLine);
};

const extractFunctionDeclaration = (func: string[]): Declaration => {
  const joined = func.join('\n');
  const declaration = joined.slice(0, joined.indexOf('{')).trim().split('\n');
  return declaration.map(formatLine);
};

const extractFunctionFirstLine = (b: Body): Line => {
  return b[0];
};

const extractFunctionLastLines = (b: Body): Line[] => {
  if (b.findIndex(l => l.returns) === -1) return [b[b.length - 1]];
  return b.filter(l => l.returns);
};

const extractFunctionEvents = (b: Body): Event[] => {
  return b.filter(l => l.emits).map(formatEvent);
};

const extractFunctionBlocks = (b: Body): Block[] => {
  const k: Block[] = [];
  for (let i = 0; i < b.length; i++) {
    const l = b[i].content.trim();
    const o = {} as Block;
    o.type = BLOCK_TYPES.filter(t => t.regex.test(l)).map(t => t.name)[0];
    if (o.type === undefined) continue;
    const start = b[i].content.indexOf(o.type);
    o.lines = [];
    for (let j = i; j < b.length; j++) {
      o.lines.push(b[j]);
      if (b[j].content.indexOf('}') === start) break;
    }
    k.push(o);
  }

  return k;
};

const extractFunctionParams = (d: Declaration): Variable[] => {
  const j = d.map(l => l.content).join('\n');
  const p = j
    .slice(j.indexOf('(') + 1, j.indexOf(')'))
    .trim()
    .split(', ')
    .join('\n');
  const t = p
    .split('\n')
    .map(x => x.replace(',', '').trim())
    .filter(x => x != '');
  return t.map(l => formatVariable(l.split(' ')));
};

const extractFunctionModifiers = (d: Declaration): string[] => {
  const j = d.map(l => l.content).join('\n');
  const s = j.slice(j.indexOf(')') + 1, j.indexOf('returns')).trim();
  return s
    .split(' ')
    .map(x => x.trim())
    .filter(x => x != '');
};

const extractFunctionReturns = (d: Declaration): Variable[] => {
  const j = d.map(l => l.content).join('\n');
  const s = j.slice(j.indexOf('returns')).trim();
  if (s == '') return [];

  const m = s
    .replace('returns ', '')
    .replaceAll('(', '')
    .replaceAll(')', '')
    .split(', ')
    .map(x => x.trim());
  return m.map(p => formatVariable(p.split(' ')));
};

const extractFunctionSignature = (func: string[]) => {
  const f = func.map(l => l.slice(PADDING)).join('');
  const k = f.slice(
    f.indexOf('function ') + 'function '.length,
    f.indexOf(')') + 1,
  );
  const n = k.slice(0, k.indexOf('(')).trim();
  const p = k.slice(k.indexOf('(') + 1, k.indexOf(')')).trim();

  let s: string;
  if (p === '') {
    s = n + '()';
  } else {
    s =
      n +
      '(' +
      p
        .trim()
        .split(',')
        .map(x => x.trim().split(' ')[0])
        .join(',') +
      ')';
  }
  return s;
};

export const extractFunctionSelector = (s: string): string => {
  return ethers.utils.keccak256(ethers.utils.toUtf8Bytes(s)).slice(0, 10);
};

export const extractFunctionID = (s: string) => {
  return Number(s);
};

// ============= //
// |  HELPERS  | //
// ============= //

const formatVariable = (p: string[]): Variable => {
  const v = {} as Variable;
  v.type = p[0];
  v.location = SOLIDITY_STORAGE_LOCATIONS.some(x => p[1] === x) ? p[1] : p[2];
  v.name = SOLIDITY_STORAGE_LOCATIONS.some(x => p[1] === x) ? p[2] : p[1];
  return v;
};

const formatLine = (l: string): Line => {
  const line = {} as Line;
  line.number = parseInt(
    l.slice(0, PADDING).replace('L', '').replace(':', '').trim(),
  );
  line.index = line.number - 1;
  line.content = l.slice(PADDING);
  line.returns = line.content.trim().startsWith('return ');
  line.emits =
    line.content.trim().startsWith('emit ') &&
    line.content.trim().endsWith(';');
  return line;
};

const formatEvent = (l: Line): Event => {
  const e = {} as Event;
  e.line = l;
  e.name = extractEventName(l);
  return e;
};

const extractEventName = (l: Line): string => {
  return l.content.slice(
    l.content.indexOf('emit ') + 'emit '.length,
    l.content.indexOf('('),
  );
};
