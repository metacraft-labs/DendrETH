const os = require('os');
const cpus = os.cpus();
const isM1Mac = cpus[0].model.includes("Apple M1");

import { c as cTester, wasm as wasmTester } from "circom_tester";

export const c = cTester;
export const wasm = wasmTester;
export const fastestTester = isM1Mac ? wasmTester : cTester;
