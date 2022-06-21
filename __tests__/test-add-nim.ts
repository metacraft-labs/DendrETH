import { loadWasm } from '../src/ts-utils/load-wasm';

interface MyExports {
  [key: string]: WebAssembly.ExportValue;
  printAdd: (a: number, b: number) => void;
}

describe('calling Nim functions compiled to Wasm', () => {
  let logMessages: string[];
  let exports: MyExports;
  beforeEach(async () => {
    logMessages = [];
    exports = await loadWasm<MyExports>({
      from: {
        filepath: './src/nimToWasm/add.wasm',
      },
      importObject: {
        env: {
          print: (x: unknown) => logMessages.push(String(x)),
        },
      },
    });
  });

  test('printAdd', () => {
    const res = exports.printAdd(2, 3);
    expect(res).toBe(undefined);
    expect(logMessages).toEqual(['5']);
  });
});
