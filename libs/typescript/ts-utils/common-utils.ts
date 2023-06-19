import * as fs from 'fs';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';
import { sha256 } from 'ethers/lib/utils';

const exec = promisify(exec_);

export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
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

export function getEnvString(varName: string) {
  return assertNotNull(
    process.env[varName],
    `Env variable '${varName}' is missing.`,
  );
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
