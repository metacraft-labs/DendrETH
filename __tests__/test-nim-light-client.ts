import { dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import glob_ from 'glob';
const glob = glob_.sync;
import { ssz } from '@lodestar/types';

import { compileNimFileToWasm } from '../src/ts-utils/compile-nim-to-wasm';
import {
  loadWasm,
  marshalSzzObjectToWasm,
  WasmError,
  wasmException,
} from '../src/ts-utils/wasm-utils';
import { hexToArray, arrayToString } from '../src/ts-utils/hex-utils';
import { SSZSpecTypes } from '../src/ts-utils/sszSpecTypes';

import BOOTSTRAP from './bootstrap.json';

interface NimTestState<T extends WebAssembly.Exports = {}> {
  exports: T;
  logMessages: string[];
  wasmFilePath: string;
}

describe('Light Client in Nim compiled to Wasm', () => {
  const filesToTest = glob(
    dirname(fileURLToPath(import.meta.url)) + '/nimLightClient/*.nim',
    {
      ignore: '**/panicoverride\\.nim',
    },
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
              wasmQuit: (x: any, y: any) => {
                {
                  throw wasmException(x, y);
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
  }, 20000);

  testNimToWasmFile<{
    assertLCFailTest: (a: number) => any;
    memory: WebAssembly.Memory;
  }>(
    'Test triggering `assertLC` in nim module',
    'assertLCTest.nim',
    ({ exports, logMessages }) => {
      expect(() => {
        exports.assertLCFailTest(42);
      }).toThrow();

      try {
        exports.assertLCFailTest(42);
      } catch (err) {
        const error = new Uint8Array(
          exports.memory.buffer,
          (err as WasmError).errMessageOffset,
          (err as WasmError).errSize,
        );
        expect(arrayToString(error)).toStrictEqual('Invalid Block');
      }
    },
  );


  testNimToWasmFile<{
    eth2DigestCompare: (a: Uint8Array) => Boolean;
    memory: WebAssembly.Memory;
  }>('Compare eth2Digests', 'eth2Digest.nim', ({ exports, logMessages }) => {
    const correctBlockRoot =
      'ca6ddab42853a7aef751e6c2bf38b4ddb79a06a1f971201dcf28b0f2db2c0d61';
    const correctBlockRootBuffer = hexToArray(correctBlockRoot);
    var blockRootArray = new Uint8Array(exports.memory.buffer, 0, 32);
    blockRootArray.set(correctBlockRootBuffer);
    const correctTestRes = exports.eth2DigestCompare(blockRootArray);
    expect(correctTestRes).toStrictEqual(1);

    const incorrectBlockRoot =
      'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    const incorrectBlockRootBuffer = hexToArray(incorrectBlockRoot);
    var blockRootArray = new Uint8Array(exports.memory.buffer, 0, 32);
    blockRootArray.set(incorrectBlockRootBuffer);
    const incorrectTestRes = exports.eth2DigestCompare(blockRootArray);
    expect(incorrectTestRes).toStrictEqual(0);
  });

  testNimToWasmFile<{
    allocMemory: (a: number) => any;
    beaconBlockHeaderCompare: (a: number, b: number) => any;
    memory: WebAssembly.Memory;
  }>(
    'Compare Beacon Block Headers',
    'beaconBlockHeader.nim',
    ({ exports, logMessages }) => {
      const testBeaconBlockHeader = ssz.phase0.BeaconBlockHeader.fromJson({
        slot: 3566048,
        proposer_index: 265275,
        parent_root:
          '0x6d8394a7292d616d8825f139b09fc4dca581a9c0af44499b3283b2dfda346762',
        state_root:
          '0x2176c5be4719af3e4ac67e12c55e273468791becc5ee60b4e430a05fd289acdd',
        body_root:
          '0x916babc5bb75209f7a279ed8dd2545721ea3d6b2b6ab331c74dd4247db172b8b',
      });

      const { startOffset, length } = marshalSzzObjectToWasm(
        exports,
        testBeaconBlockHeader,
        ssz.phase0.BeaconBlockHeader,
      );

      expect(
        exports.beaconBlockHeaderCompare(startOffset, length),
      ).toStrictEqual(1);
    },
  );

  testNimToWasmFile<{
    allocMemory: (a: number) => any;
    initializeLightClientStoreTest: (a: number, b: number) => any;
    memory: WebAssembly.Memory;
  }>(
    'Test `initialize_light_client_store`',
    'initializeLightClientStore.nim',
    ({ exports, logMessages }) => {
      const header = ssz.phase0.BeaconBlockHeader.fromJson(
        BOOTSTRAP.data.v.header,
      );
      const { startOffset: headerStartOffset, length: headerLength } =
        marshalSzzObjectToWasm(exports, header, ssz.phase0.BeaconBlockHeader);

      const bootstrap = SSZSpecTypes.LightClientBootstrap.fromJson(
        BOOTSTRAP.data.v,
      );
      const { startOffset: bootstrapStartOffset, length: bootstrapLength } =
        marshalSzzObjectToWasm(
          exports,
          bootstrap,
          SSZSpecTypes.LightClientBootstrap,
        );
      expect(
        exports.initializeLightClientStoreTest(
          headerStartOffset,
          bootstrapStartOffset,
        ),
      ).toStrictEqual(1);
    },
  );
});
