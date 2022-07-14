import { loadWasm } from '../src/ts-utils/load-wasm';

interface MyExports {
  [key: string]: WebAssembly.ExportValue;
  printCreateSeqLen: (a: number, b: number) => void;
}

describe('calling Nim functions compiled to Wasm', () => {
  let logMessages: string[];
  let exports: MyExports;
  beforeEach(async () => {
    logMessages = [];
    exports = await loadWasm<MyExports>({
      from: {
        filepath: './src/nimToWasm/seq_append.wasm',
      },
      importObject: {
        env: {
          print: (x: unknown) => logMessages.push(String(x)),
        },
      },
    });
  });

  test('printCreateSeqLen', () => {
    const res = exports.printCreateSeqLen(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });
});
