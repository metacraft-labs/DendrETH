import { dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';


import glob_ from 'glob';
const glob = glob_.sync;

import { compileNimFileToWasm } from '../src/ts-utils/compile-nim-to-wasm';
import { loadWasm } from '../src/ts-utils/load-wasm';

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
    path: string,
    func: (state: NimTestState<T>) => void,
  ) {
    test(`Testing '${path}'`, () =>
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
  });

  testNimToWasmFile<{
    printAdd: (a: number, b: number) => void;
  }>('add.nim', ({ exports, logMessages }) => {
    const res = exports.printAdd(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });

  testNimToWasmFile<{
    printCreateSeqLen: (a: number, b: number) => void;
  }>('seq_append.nim', ({ exports, logMessages }) => {
    const res = exports.printCreateSeqLen(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });
});
