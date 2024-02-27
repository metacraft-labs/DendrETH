import { log } from '../logging';

export interface LevelIndexAndGIndex {
  [key: string]: bigint;
}
export type Tasks = Record<`${bigint}`, Promise<unknown>>;

export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createArrayFromRange(x: number, y: number) {
  return Array.from({ length: y - x + 1 }, (_, index) => x + index);
}

export function stringify(obj: unknown) {
  return JSON.stringify(obj, (_, value) =>
    typeof value === 'bigint' ? value.toString() : value,
  );
}

export function stringToBytes(str: string) {
  const bytes = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) {
    bytes[i] = str.charCodeAt(i);
  }
  return bytes;
}

let logWrites = true;

export function setLogging(enabled: boolean) {
  logWrites = enabled;
}

export function logWrite(gIndex: bigint, msg: string) {
  if (logWrites) log(msg, gIndex);
}
