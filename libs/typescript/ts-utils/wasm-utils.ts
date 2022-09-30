import { readFile } from 'fs/promises';
import '@chainsafe/ssz';
import { Type } from '@chainsafe/ssz';
import { assert } from 'node:console';

var HEAP,
  /** @type {ArrayBuffer} */
  buffer,
  /** @type {Int8Array} */
  HEAP8,
  /** @type {Uint8Array} */
  HEAPU8: Uint8Array,
  /** @type {Int16Array} */
  HEAP16,
  /** @type {Uint16Array} */
  HEAPU16,
  /** @type {Int32Array} */
  HEAP32,
  /** @type {Uint32Array} */
  HEAPU32,
  /** @type {Float32Array} */
  HEAPF32,
  /** @type {Float64Array} */
  HEAPF64;

function alignUp(x: any, multiple: any) {
  if (x % multiple > 0) {
    x += multiple - (x % multiple);
  }
  return x;
}
var Module: any = typeof Module !== 'undefined' ? Module : {};

function updateGlobalBufferAndViews(buf: any) {
  buffer = buf;
  Module['HEAPU8'] = HEAP8 = new Uint8Array(buf);
}

export function emscripten_realloc_buffer(
  size: any,
  memory: WebAssembly.Memory,
) {
  try {
    // round size grow request up to wasm page size (fixed 64KB per spec)
    memory.grow((size - memory.buffer.byteLength + 65535) >>> 16); // .grow() takes a delta compared to the previous size
    updateGlobalBufferAndViews(memory.buffer);
    return 1 /*success*/;
  } catch (e) {
    console.log(
      'emscripten_realloc_buffer: Attempted to grow heap from ' +
        memory.buffer.byteLength +
        ' bytes to ' +
        size +
        ' bytes, but got error: ' +
        e,
    );
  }
  // implicit 0 return to save code size (caller will cast "undefined" into 0
  // anyhow)
}

export async function loadWasm<Exports extends WebAssembly.Exports>({
  from,
  importObject,
}: {
  from: { url: URL } | { filepath: string };
  importObject: WebAssembly.Imports;
}) {
  let res: WebAssembly.WebAssemblyInstantiatedSource;
  let memory: WebAssembly.Memory;
  let HEAPU8: Uint8Array;
  importObject = {
    env: {
      print: (x: undefined) => console.log(x),
      // __main_argc_argv is needed for the new version of emscripten
      __main_argc_argv: () => console.log('__main_argc_argv'),
      exit: () => console.log('exit'),
      wasmQuit: (errOffset: number, errLength: number) => {
        throwWasmException({
          memory,
          startOffset: errOffset,
          length: errLength,
        });
      },
      emscripten_notify_memory_growth: (x: undefined) =>
        console.log('emscripten_notify_memory_growth'),
      emscripten_memcpy_big: (dest: any, src: any, num: any) => {
        HEAPU8.copyWithin(dest, src, src + num);
      },
      emscripten_resize_heap: (requestedSize: any) => {
        console.log('emscripten_resize_heap');
        var oldSize = HEAPU8.length;
        requestedSize = requestedSize >>> 0;
        // With pthreads, races can happen (another thread might increase the size in between), so return a failure, and let the caller retry.
        assert(requestedSize > oldSize);

        // Memory resize rules:
        // 1. Always increase heap size to at least the requested size, rounded up to next page multiple.
        // 2a. If MEMORY_GROWTH_LINEAR_STEP == -1, excessively resize the heap geometrically: increase the heap size according to
        //                                         MEMORY_GROWTH_GEOMETRIC_STEP factor (default +20%),
        //                                         At most overreserve by MEMORY_GROWTH_GEOMETRIC_CAP bytes (default 96MB).
        // 2b. If MEMORY_GROWTH_LINEAR_STEP != -1, excessively resize the heap linearly: increase the heap size by at least MEMORY_GROWTH_LINEAR_STEP bytes.
        // 3. Max size for the heap is capped at 2048MB-WASM_PAGE_SIZE, or by MAXIMUM_MEMORY, or by ASAN limit, depending on which is smallest
        // 4. If we were unable to allocate as much memory, it may be due to over-eager decision to excessively reserve due to (3) above.
        //    Hence if an allocation fails, cut down on the amount of excess growth, in an attempt to succeed to perform a smaller allocation.

        // A limit is set for how much we can grow. We should not exceed that
        // (the wasm binary specifies it, so if we tried, we'd fail anyhow).
        // In CAN_ADDRESS_2GB mode, stay one Wasm page short of 4GB: while e.g. Chrome is able to allocate full 4GB Wasm memories, the size will wrap
        // back to 0 bytes in Wasm side for any code that deals with heap sizes, which would require special casing all heap size related code to treat
        // 0 specially.
        var maxHeapSize = 2147483648;
        if (requestedSize > maxHeapSize) {
          console.log(
            'Cannot enlarge memory, asked to go up to ' +
              requestedSize +
              ' bytes, but the limit is ' +
              maxHeapSize +
              ' bytes!',
          );
          return false;
        }

        // Loop through potential heap size increases. If we attempt a too eager reservation that fails, cut down on the
        // attempted size and reserve a smaller bump instead. (max 3 times, chosen somewhat arbitrarily)
        for (var cutDown = 1; cutDown <= 4; cutDown *= 2) {
          var overGrownHeapSize = oldSize * (1 + 0.2 / cutDown); // ensure geometric growth
          // but limit overreserving (default to capping at +96MB overgrowth at most)
          overGrownHeapSize = Math.min(
            overGrownHeapSize,
            requestedSize + 100663296,
          );

          var newSize = Math.min(
            maxHeapSize,
            alignUp(Math.max(requestedSize, overGrownHeapSize), 65536),
          );

          var replacement = emscripten_realloc_buffer(newSize, memory);
          if (replacement) {
            return true;
          }
        }
        console.log(
          'Failed to grow the heap from ' +
            oldSize +
            ' bytes to ' +
            ' bytes, not enough memory!',
        );
        return false;
      },

      blscurve_blst_min_pubkey_sig_coreInit000: (x: any) => {
        'blscurve_blst_min_pubkey_sig_coreInit000';
      },
      main: (x: any) => {
        'main';
      },

      ...importObject.env,
    },
    wasi_snapshot_preview1: {
      args_sizes_get: (x: any) => {
        'args_sizes_get';
      },
      args_get: (x: any) => {
        'args_get';
      },
      proc_exit: (x: any) => {
        'proc_exit';
      },
      clock_res_get: () => console.log('clock_res_get'),
      fd_fdstat_get: () => console.log('fd_fdstat_get'),
      clock_time_get: () => console.log('clock_time_get'),
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
  HEAPU8 = new Uint8Array(memory.buffer);
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

// Write ssz representations of some object in the wasm memory.
// Serialize the object, allocate memory and write it
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

// Write ssz representations of some object in the wasm memory.
// Serialize the object and write it on previously calculated offset
export function writeSSZObjectToWasm<T>(
  { memory }: WasmModuleMemoryInterface,
  sszObject: T,
  sszType: Type<T>,
  startOffset,
): MemorySlice {
  const serialized = sszType.serialize(sszObject);
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

export async function readJson(path: string): Promise<unknown> {
  return JSON.parse(await readFile(path, 'utf-8'));
}
