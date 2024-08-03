import * as fs from 'fs';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import { sha256 } from 'ethers/lib/utils';
import { assert } from 'console';

const exec = promisify(exec_);

export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function gindexFromIndex(index: bigint, depth: bigint): bigint {
  return 2n ** depth + index;
}

export function indexFromGindex(gindex: bigint, depth: bigint): bigint {
  return gindex - 2n ** depth;
}

export function getDepthByGindex(gindex: number): number {
  return Math.floor(Math.log2(gindex));
}

export function getNthParent(gindex: bigint, n: bigint): bigint {
  return gindex / 2n ** n;
}

function log2bigInt(n: bigint): bigint {
  return BigInt(n.toString(2).length) - 1n;
}

export function concatGIndices(...gindices: bigint[]) {
  let firstTreeGIndex = gindices[0];
  for (let i = 1; i < gindices.length; ++i) {
    const secondTreeGIndex = gindices[i];
    const firstTreeDepth = log2bigInt(firstTreeGIndex);
    const secondTreeDepth = log2bigInt(secondTreeGIndex);
    const bigTreeDepth = firstTreeDepth + secondTreeDepth;

    const leavesToSkip = firstTreeGIndex - 2n ** firstTreeDepth;
    const indicesPerLeaf = 2n ** secondTreeDepth;
    const firstLeafGIndexInBigTree = 2n ** bigTreeDepth;
    const firstSubtreeLeafGIndex =
      firstLeafGIndexInBigTree + leavesToSkip * indicesPerLeaf;
    const indexInSubtree = secondTreeGIndex - 2n ** secondTreeDepth;
    firstTreeGIndex = firstSubtreeLeafGIndex + indexInSubtree;
  }

  return firstTreeGIndex;
}

export function getParent(gindex: bigint): bigint {
  return getNthParent(gindex, 1n);
}

export function getLastSlotInEpoch(epoch: bigint): bigint {
  return epoch * 32n + 31n;
}

export function getFirstSlotInEpoch(epoch: bigint): bigint {
  return epoch * 32n;
}

// TODO: make indices be a number[]
export function* makeBranchIterator(indices: bigint[], depth: bigint) {
  const changedValidatorGindices = indices.map(index =>
    gindexFromIndex(index, depth),
  );

  yield changedValidatorGindices;

  let nodesNeedingUpdate = new Set(changedValidatorGindices.map(getParent));
  while (nodesNeedingUpdate.size !== 0) {
    const newNodesNeedingUpdate = new Set<bigint>();

    for (const gindex of nodesNeedingUpdate) {
      if (gindex !== 1n) {
        newNodesNeedingUpdate.add(getParent(gindex));
      }
    }

    yield [...nodesNeedingUpdate];
    nodesNeedingUpdate = newNodesNeedingUpdate;
  }
}

export function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += 2 ** (7 - bit) * hash[byte * 8 + bit];
    }
    result[byte] = value;
  }
  return result;
}

export function byteArrayToNumber(byteArray) {
  var value = 0;
  for (var i = byteArray.length - 1; i >= 0; i--) {
    value = value * 256 + byteArray[i];
  }
  return value;
}

export function checkConfig(config: any) {
  for (const envVar of Object.keys(config)) {
    if (!config[envVar]) {
      console.warn(`$${envVar} environment variable is not set`);
      process.exit(0);
    }
  }
}

// Open Json file and append data to it
export function appendJsonFile(filePath: string, data: any) {
  let fileData: any[] = [];
  try {
    fileData = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (e) {
    console.warn(`Unable to read file ${filePath}`);
  }
  fileData.push(data);
  fs.writeFileSync(filePath, JSON.stringify(fileData, null, 2));
}

export async function getRootDir() {
  return (await exec('git rev-parse --show-toplevel')).stdout.replace(
    /\s/g,
    '',
  );
}

export function assertNotNull<T>(
  value: T | null | undefined,
  errorMessage?: string,
): T {
  if (
    value === null ||
    value === undefined ||
    (typeof value === 'string' && !value.length)
  ) {
    throw new Error(errorMessage ?? 'Assertion failed: value is null');
  }
  return value;
}

export function checkIfNull(value: string | null | undefined, msg: string) {
  if (
    value === null ||
    value === undefined ||
    (typeof value === 'string' && !value.length)
  ) {
    console.log(msg);
    return true;
  }
  return false;
}

export function getEnvString(varName: string) {
  return assertNotNull(
    process.env[varName],
    `Env variable '${varName}' is missing.`,
  );
}

export function getSecretEnvString(varName: string) {
  if (
    checkIfNull(process.env[varName], `Env variable '${varName}' is missing.`)
  ) {
    return '';
  } else {
    let path = process.env[varName];
    const rootDir = getEnvString('GIT_ROOT');
    return fs.readFileSync(rootDir + path, 'ascii').trim();
  }
}

export function unstringifyBigInts(o) {
  if (typeof o == 'string' && /^[0-9]+$/.test(o)) {
    return BigInt(o);
  } else if (typeof o == 'string' && /^0x[0-9a-fA-F]+$/.test(o)) {
    return BigInt(o);
  } else if (Array.isArray(o)) {
    return o.map(unstringifyBigInts);
  } else if (typeof o == 'object') {
    if (o === null) return null;
    const res = {};
    const keys = Object.keys(o);
    keys.forEach(k => {
      res[k] = unstringifyBigInts(o[k]);
    });
    return res;
  } else {
    return o;
  }
}

function hexToBitString(num) {
  var bitmask: string = '';
  switch (num) {
    case '0': {
      bitmask = '0000';
      break;
    }
    case '1': {
      bitmask = '0001';
      break;
    }
    case '2': {
      bitmask = '0010';
      break;
    }
    case '3': {
      bitmask = '0011';
      break;
    }
    case '4': {
      bitmask = '0100';
      break;
    }
    case '5': {
      bitmask = '0101';
      break;
    }
    case '6': {
      bitmask = '0110';
      break;
    }
    case '7': {
      bitmask = '0111';
      break;
    }
    case '8': {
      bitmask = '1000';
      break;
    }
    case '9': {
      bitmask = '1001';
      break;
    }
    case 'a': {
      bitmask = '1010';
      break;
    }
    case 'b': {
      bitmask = '1011';
      break;
    }
    case 'c': {
      bitmask = '1100';
      break;
    }
    case 'd': {
      bitmask = '1101';
      break;
    }
    case 'e': {
      bitmask = '1110';
      break;
    }
    case 'f': {
      bitmask = '1111';
      break;
    }
  }
  return bitmask;
}

export function bitTo2BigInts(hexNum) {
  var firstNumInBits: string = '0b000';
  var secondNumInBits: string = '0b';
  for (var i = 0; i < 253; i += 1) {
    secondNumInBits = secondNumInBits.concat('0');
  }

  for (var i = 2; i < 65; i += 1) {
    firstNumInBits = firstNumInBits.concat(
      hexToBitString(sha256(hexNum)[i]).toString(),
    );
  }
  const lastBitArray = hexToBitString(sha256(hexNum)[65]);

  firstNumInBits = firstNumInBits.concat(lastBitArray[0].toString());
  secondNumInBits = secondNumInBits.concat(lastBitArray[1].toString());
  secondNumInBits = secondNumInBits.concat(lastBitArray[2].toString());
  secondNumInBits = secondNumInBits.concat(lastBitArray[3].toString());

  return [BigInt(firstNumInBits), BigInt(secondNumInBits)];
}

export function splitIntoBatches<T>(array: T[], batchSize: number): T[][] {
  const batches: T[][] = [];

  for (let i = 0; i < array.length; i += batchSize) {
    batches.push(array.slice(i, i + batchSize));
  }

  return batches;
}

export function groupBy<T, K extends string | number | symbol>(
  array: T[],
  fn: (item: T) => K,
): Record<K, T[]> {
  return array.reduce((result, item) => {
    const key = fn(item);
    (result[key] = result[key] || []).push(item);
    return result;
  }, {} as Record<K, T[]>);
}
/**
 * Executes a function repeatedly while a given condition is true.
 * @param cond A function that takes the result of the repeated function as an
 *  argument and returns a boolean value.
 * @param f The function to execute repeatedly.
 * @param time The amount of time to wait between each execution of the function.
 * @return The final result of the repeated function.
 */
export async function loopWhile<T>(
  cond: (r: T) => boolean,
  f: () => Promise<T>,
  time: number,
): Promise<T> {
  let r = await f();
  while (cond(r)) {
    await sleep(time);
    r = await f();
  }
  return r;
}

export function panic(message: string): never {
  console.error(`panicked: ${message}`);
  return process.exit(1);
}

export function getBigIntFromLimbs(limbs: number[]): bigint {
  let result = 0n;

  for (let i = 0; i < limbs.length; i++) {
    assert(
      Number.isInteger(limbs[i]) && 0 <= limbs[i] && limbs[i] <= 0xffffffff,
      'Each word must fit in uint32',
    );

    result |= BigInt(limbs[i]) << BigInt(32 * i);
  }
  return result;
}

export function range(low: number, highExclusive: number): number[] {
  const result: number[] = [];
  for (let value = low; value < highExclusive; ++value) {
    result.push(value);
  }
  return result;
}

export function extractHostnameAndPort(url: string) {
  const urlObj = new URL(url);
  const hostname = urlObj.hostname;
  const port = urlObj.port;

  return { hostname, port };
}
