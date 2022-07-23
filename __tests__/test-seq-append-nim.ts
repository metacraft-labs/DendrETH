import { loadWasm } from '../src/ts-utils/load-wasm';
import { compileNimFileToWasm } from '../src/ts-utils/compile-nim-to-wasm';

interface MyExports {
  [key: string]: WebAssembly.ExportValue;
  printCreateSeqLen: (a: number, b: number) => void;
}

const nimFilePath = './src/nimToWasm/seq_append.nim';


describe('calling Nim functions compiled to Wasm', () => {
  let logMessages: string[];
  let exports: MyExports;
  let wasmFilePath: string;
  beforeEach(async () => {
    wasmFilePath = (await compileNimFileToWasm(nimFilePath)).outputFileName;
    logMessages = [];
    exports = await loadWasm<MyExports>({
      from: {
        filepath: wasmFilePath,
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
