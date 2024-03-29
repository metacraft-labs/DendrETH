import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';
import { groupBy } from '@dendreth/utils/ts-utils/common-utils';
import { formatHex } from '@dendreth/utils/ts-utils/bls';

const sszTestCasesDir = path.resolve(
  __dirname,
  '../../../vendor/eth2.0-tests/ssz',
);

// circom prime is 254 bits
const testCasesBounds = readTestCases(`${sszTestCasesDir}/uint_bounds.yaml`)
  .filter(x => x.valid)
  .filter(x => getBitsNumberFromType(x.type) <= 248);

const testCasesRandom = readTestCases(`${sszTestCasesDir}/uint_random.yaml`)
  .filter(x => x.valid)
  .filter(x => getBitsNumberFromType(x.type) <= 248);

const testCasesArray = [...testCasesBounds, ...testCasesRandom];

const groupedTestCases = groupBy(testCasesArray, x => x.type);

for (const [type, testCaseGroup] of Object.entries(groupedTestCases) as [
  keyof typeof groupedTestCases,
  TestCase[],
][]) {
  fs.mkdirSync(`ssz_num_${type}`, { recursive: true });
  fs.mkdirSync(`ssz_num_${type}/data`, { recursive: true });

  fs.writeFileSync(`ssz_num_${type}/circuit.circom`, getCircomTemplate(type));

  const fileCountTracker: Record<string, number> = {};

  for (const testCase of testCaseGroup) {
    const folderName = getFolderName(type, testCase.tags[2], fileCountTracker);

    fs.mkdirSync(folderName, { recursive: true });

    fs.writeFileSync(
      `${folderName}/input.json`,
      JSON.stringify({ in: testCase.value }),
    );

    fs.writeFileSync(
      `${folderName}/output.json`,
      JSON.stringify({
        out: BigInt('0x' + formatHex(testCase.ssz))
          .toString(2)
          .padStart(getBitsNumberFromType(type), '0')
          .padEnd(256, '0')
          .split(''),
      }),
    );
  }
}

function getCircomTemplate(type: TestCase['type']) {
  return `pragma circom 2.1.5;

include "../../circuits/ssz_num.circom";

component main = SSZNum(${getBitsNumberFromType(type)});`;
}

function getBitsNumberFromType(type: TestCase['type']): number {
  return Number.parseInt(type.replace('uint', ''));
}

interface TestCase {
  type:
    | 'uint8'
    | 'uint16'
    | 'uint24'
    | 'uint32'
    | 'uint40'
    | 'uint48'
    | 'uint56'
    | 'uint64'
    | 'uint72'
    | 'uint80'
    | 'uint88'
    | 'uint96'
    | 'uint104'
    | 'uint112'
    | 'uint120'
    | 'uint128'
    | 'uint136'
    | 'uint144'
    | 'uint152'
    | 'uint160'
    | 'uint168'
    | 'uint176'
    | 'uint184'
    | 'uint192'
    | 'uint200'
    | 'uint208'
    | 'uint216'
    | 'uint224'
    | 'uint232'
    | 'uint240'
    | 'uint248'
    | 'uint256'
    | 'uint264'
    | 'uint272'
    | 'uint280'
    | 'uint288'
    | 'uint296'
    | 'uint304'
    | 'uint312'
    | 'uint320'
    | 'uint328'
    | 'uint336'
    | 'uint344'
    | 'uint352'
    | 'uint360'
    | 'uint368'
    | 'uint376'
    | 'uint384'
    | 'uint392'
    | 'uint400'
    | 'uint408'
    | 'uint416'
    | 'uint424'
    | 'uint432'
    | 'uint440'
    | 'uint448'
    | 'uint456'
    | 'uint464'
    | 'uint472'
    | 'uint480'
    | 'uint488'
    | 'uint496'
    | 'uint504'
    | 'uint512';
  valid: boolean;
  value: string;
  ssz: string;
  tags: string[];
}

function readTestCases(filePath: string): TestCase[] {
  const fileContent = fs.readFileSync(filePath, 'utf8');
  const obj = yaml.load(fileContent) as any;

  return obj.test_cases as TestCase[];
}

function getNextFileCount(
  tag: string,
  fileCountTracker: Record<string, number>,
): number {
  fileCountTracker[tag] = (fileCountTracker[tag] || 0) + 1;

  return fileCountTracker[tag];
}

function getFolderName(
  type: string,
  tag: string,
  fileCountTracker: Record<string, number>,
): string {
  const baseName = `ssz_num_${type}/data/${tag}`;
  const count = getNextFileCount(baseName, fileCountTracker);

  return count > 1 ? `${baseName}${count}` : baseName;
}
