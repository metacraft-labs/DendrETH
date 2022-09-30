import * as fs from 'fs';
import * as path from 'path';

export const CONTRACTS_ORIG_PATH = path.join(
  __dirname,
  '..',
  '..',
  '..',
  '..',
  'contracts',
);
export const CONTRACTS_TEMP_PATH = path.join(
  __dirname,
  '..',
  '..',
  '..',
  '..',
  'contracts',
  'temp',
);
export const REPORT_PATH = path.join(__dirname, '..', 'output', 'report.txt');
export const SOLIDITY_STORAGE_LOCATIONS = ['calldata', 'memory', 'storage'];
export const GAS_REPORTABLE = fs.readFileSync(
  path.join(__dirname, '..', 'templates', 'GasReportable.sol'),
);
export const PADDING = 8;
export const BLOCK_TYPES = [
  {
    regex: /^for\b/g,
    name: 'for',
  },
  {
    regex: /^while\b/g,
    name: 'while',
  },
  {
    regex: /^unchecked\b/g,
    name: 'unchecked',
  },
  {
    regex: /^assembly\b/g,
    name: 'assembly',
  },
];
