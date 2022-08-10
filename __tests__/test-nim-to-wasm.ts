import { dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import glob_ from 'glob';
const glob = glob_.sync;

import { compileNimFileToWasm } from '../src/ts-utils/compile-nim-to-wasm';
import { loadWasm, marshalSzzObjectToWasm } from '../src/ts-utils/wasm-utils';
import { hexToArray } from  '../src/ts-utils/hex-utils';
interface NimTestState<T extends WebAssembly.Exports = {}> {
  exports: T;
  logMessages: string[];
  wasmFilePath: string;
}

describe('calling Nim functions compiled to Wasm', () => {
  const filesToTest = glob(
    dirname(fileURLToPath(import.meta.url)) + '/nimToWasm/*.nim', {
      ignore: '**/panicoverride\\.nim'
    }
  );

  const perFileState: Record<string, NimTestState> = {};

  function testNimToWasmFile<T extends WebAssembly.Exports = {}>(
    testName: string,
    path: string,
    func: (state: NimTestState<T>) => void,
  ) {
    test(`Testing '${path}': '${testName}'`, () =>
      func(perFileState[path] as NimTestState<T>));
  }

  beforeAll(async () => {
    await Promise.all(
      filesToTest.map(async nimFilePath => {
        const wasmFilePath = (await compileNimFileToWasm(nimFilePath))
          .outputFileName;
        const exports = await loadWasm<{}>({
          from: {
            filepath: wasmFilePath,
          },
          importObject: {
            env: {
              print: (x: unknown) =>
                perFileState[basename(nimFilePath)].logMessages.push(String(x)),
              wasmQuit: (x: any, y: any) => {
                  {
                    throw("");
                  }
                },
            },
          },
        });
        perFileState[basename(nimFilePath)] = {
          wasmFilePath,
          logMessages: [],
          exports,
        };
      }),
    );
  }, 20000 /* timeout in milliseconds */);

  testNimToWasmFile<{
    printAdd: (a: number, b: number) => void;
  }>("Sum of two numbers", 'add.nim', ({ exports, logMessages }) => {
    const res = exports.printAdd(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });

  testNimToWasmFile<{
    printCreateSeqLen: (a: number, b: number) => void;
  }>("Length of seq", 'seq_append.nim', ({ exports, logMessages }) => {
    const res = exports.printCreateSeqLen(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });

  testNimToWasmFile<{
    sumOfArrayElements: (a: Int32Array) => number;
    memory:WebAssembly.Memory
  }>("Passing arrays to wasm(nim)", 'arrays.nim', ({ exports }) => {
    const array = new Int32Array(exports.memory.buffer, 0, 5);
    array.set([3, 15, 18, 4, 2]);
    let expectedRes = 42; // 3+15+18+4+2=42
    const res = exports.sumOfArrayElements(array);
    expect(res).toBe(expectedRes);
  });

  testNimToWasmFile<{
    createNewArray: (a: number) => any;
    memory:WebAssembly.Memory
  }>("Receiving arrays from wasm(nim)", 'arrays.nim', ({ exports }) => {
    let value = 42
    const expectedRes = new Int32Array(exports.memory.buffer, 0, 5)
    expectedRes.set([42, 42, 42, 42, 42]);
    let res = new Int32Array(
      exports.memory.buffer,
      exports.createNewArray(value),
      5);
    expect(res).toStrictEqual(expectedRes);
  });

  testNimToWasmFile<{
    arrayMapAdd: (a: Int32Array, b: number) => any;
    memory:WebAssembly.Memory
  }>("Passing and receiving arrays from wasm(nim)", 'arrays.nim', ({ exports }) => {
    const array = new Int32Array(exports.memory.buffer, 0, 5);
    array.set([3, 15, 18, 4, 2]);
    let value = 42
    const expectedRes = new Int32Array(exports.memory.buffer, 0, 5)
    expectedRes.set([45, 67, 60, 46, 44]);
    let res = new Int32Array(
      exports.memory.buffer,
      exports.arrayMapAdd(array, value),
      5);
    expect(res).toStrictEqual(expectedRes);
  });


testNimToWasmFile<{
  eth2DigestCompare: (a: Uint8Array) => Boolean;
  memory:WebAssembly.Memory
}>("Compare eth2Digests", 'eth2Digest.nim', ({ exports, logMessages }) => {
  const correctBlockRoot = 'ca6ddab42853a7aef751e6c2bf38b4ddb79a06a1f971201dcf28b0f2db2c0d61'
  const correctBlockRootBuffer = hexToArray(correctBlockRoot)
  var blockRootArray = new Uint8Array(exports.memory.buffer, 0, 32)
  blockRootArray.set(correctBlockRootBuffer)
  const correctTestRes = exports.eth2DigestCompare(blockRootArray);
  expect(correctTestRes).toStrictEqual(1);

  const incorrectBlockRoot = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
  const incorrectBlockRootBuffer = hexToArray(incorrectBlockRoot)
  var blockRootArray = new Uint8Array(exports.memory.buffer, 0, 32)
  blockRootArray.set(incorrectBlockRootBuffer)
  const incorrectTestRes = exports.eth2DigestCompare(blockRootArray);
  expect(incorrectTestRes).toStrictEqual(0);
  });
});
