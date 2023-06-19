import * as fs from 'fs';
import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

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

export function splitIntoBatches<T>(array: T[], batchSize: number): T[][] {
  const batches: T[][] = [];

  for (let i = 0; i < array.length; i += batchSize) {
    batches.push(array.slice(i, i + batchSize));
  }

  return batches;
}
