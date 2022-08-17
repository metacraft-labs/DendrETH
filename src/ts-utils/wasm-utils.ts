import { readFile } from 'fs/promises';
import '@chainsafe/ssz';
import { Type } from '@chainsafe/ssz';

export async function loadWasm<Exports extends WebAssembly.Exports>({
  from,
  importObject,
}: {
  from: { url: URL } | { filepath: string };
  importObject?: WebAssembly.Imports;
}) {
  let res: WebAssembly.WebAssemblyInstantiatedSource;
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
  return res.instance.exports as Exports;
}

interface WasmModuleMemoryInterface extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  allocMemory: (a: number) => any;
}

interface MemorySlice {
  startOffset: number;
  length: number;
}

export function marshalSzzObjectToWasm<T>(
  exports: WasmModuleMemoryInterface,
  sszObject: T,
  sszType: Type<T>): MemorySlice {
    const serialized = sszType.serialize(sszObject);
    const startOffset = exports.allocMemory(serialized.length);
    const slice = new Uint8Array(exports.memory.buffer);
    slice.set(serialized, startOffset);
    return {
      startOffset,
      length: serialized.length
    }
}

export interface WasmError extends Error {
  errMessageOffset: number
  errSize: number;
}

export function wasmException(errMessageOffset: any, errSize: any): WasmError {
  return {errMessageOffset: errMessageOffset, errSize, name: '', message: ''};
}
