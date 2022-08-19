import { readFile } from 'fs/promises';
import '@chainsafe/ssz';
import { Type } from '@chainsafe/ssz';

export async function loadWasm<Exports extends WebAssembly.Exports>({
  from,
  importObject,
}: {
  from: { url: URL } | { filepath: string };
  importObject: WebAssembly.Imports;
}) {
  let res: WebAssembly.WebAssemblyInstantiatedSource;
  let memory: WebAssembly.Memory;
  importObject = {
    env: {
      print: (x: unknown) => console.log(x),
      wasmQuit: (errOffset: number, errLength: number) => {
        throwWasmException({
          memory,
          startOffset: errOffset,
          length: errLength,
        });
      },
      ...importObject.env,
    },
  };

  if ('url' in from && from.url instanceof URL) {
    const url = from.url;
    const resp = fetch(url);
    res = await WebAssembly.instantiateStreaming(resp, importObject);
  } else if ('filepath' in from && typeof from.filepath === 'string') {
    // TODO: use fetch + WebAssembly.instantiateStreaming when it's supported
    // to merge this with the above implementation
    const url = new URL(from.filepath, import.meta.url + '/../../..');
    const bytes = await readFile(url);
    res = await WebAssembly.instantiate(bytes, importObject);
  } else {
    throw new Error('Invalid argument: `from`');
  }
  memory = (res.instance.exports as any).memory;
  return res.instance.exports as Exports;
}

export interface WasmModuleMemoryInterface extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  allocMemory: (a: number) => any;
}

export interface Slice {
  startOffset: number;
  length: number;
}

export interface MemorySlice extends Slice {
  memory: WebAssembly.Memory;
}

export function readMemory(slice: MemorySlice): Uint8Array {
  return new Uint8Array(slice.memory.buffer, slice.startOffset, slice.length);
}

export function writeMemory(memory: MemorySlice, bytes: Uint8Array): void {
  readMemory(memory).set(bytes, 0);
}

export function marshalSzzObjectToWasm<T>(
  { memory, allocMemory }: WasmModuleMemoryInterface,
  sszObject: T,
  sszType: Type<T>,
): MemorySlice {
  const serialized = sszType.serialize(sszObject);
  const startOffset = allocMemory(serialized.length);
  const slice: MemorySlice = {
    memory,
    startOffset,
    length: serialized.length,
  };
  writeMemory(slice, serialized);
  return slice;
}

export function decodeUtf8(slice: MemorySlice): string {
  const decoder = new TextDecoder('utf-8');
  return decoder.decode(readMemory(slice));
}

export class WasmError extends Error {}

export function throwWasmException(slice: MemorySlice): never {
  const errorMessage = decodeUtf8(slice);
  throw new WasmError(errorMessage);
}
