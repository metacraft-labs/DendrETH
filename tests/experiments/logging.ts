/* eslint-disable @typescript-eslint/no-var-requires */
import { relative } from 'path';

import { Console } from 'console';

let currentConsole: (typeof globalThis)['console'];

const disableStackCapture = true;

let color: {
  gray: (s: string) => string;
  yellow: (s: string) => string;
  bold: (s: string) => string;
  bgGray: (s: string) => string;
};

declare const window: typeof globalThis;
if (typeof window === 'undefined') {
  // Initialize console colors library:
  // color = import('colors/safe');
  color = {
    gray: s => '\x1b[90m' + s + '\x1b[0m',
    yellow: s => '\x1b[33m' + s + '\x1b[0m',
    bold: s => '\x1b[1m' + s + '\x1b[0m',
    bgGray: s => '\x1b[100m' + s + '\x1b[0m',
  };

  // Create a new Console instance, so we can force output to stdout/stderr
  // const { Console } = require('console');
  currentConsole = new Console({
    colorMode: true,
    stdout: process.stdout,
    stderr: process.stderr,
  });
} else {
  color = {
    gray: s => '\x1b[90m' + s + '\x1b[0m',
    yellow: s => '\x1b[33m' + s + '\x1b[0m',
    bold: s => '\x1b[1m' + s + '\x1b[0m',
    bgGray: s => '\x1b[100m' + s + '\x1b[0m',
  };
  currentConsole = globalThis.console;
}

const { log: defaultLog, table, error } = currentConsole;

currentConsole.log = log;

export { log, error as logError, table as logTable };

const startTime = new Date().getTime();
let prevTime = startTime;

function log<Args extends unknown[]>(msg: string, ...args: Args): void {
  const info = getCallStackInfo();
  let loc = '';
  if (info) {
    const { path, line, col } = info;
    const relativePath = relative(process.cwd(), path);
    loc = `./${relativePath}:${line}:${col}`;
  }
  const now = new Date();
  const diff = `+${(now.getTime() - startTime).toString(10)}`.padStart(5);
  const diff2 = `+${(now.getTime() - prevTime).toString(10)}`.padStart(5);
  prevTime = now.getTime();
  const whitespace = info?.inConsoleClass === true ? '\n' : ' ';
  // const prefix =
  //   `[ ${color.gray(now.toISOString())} | Δt₀: ${color.yellow(diff)} ms | ` +
  //   `Δtᵢ: ${color.yellow(diff2)} ms | ${color.bgGray(loc)} ]:` +
  //   `${whitespace}${color.bold(msg)}`;
  const prefix =
    `[ Δt₀: ${color.yellow(diff)} ms | ` +
    `Δtᵢ: ${color.yellow(diff2)} ms ]:` +
    `${whitespace}${color.bold(msg)}`;
  defaultLog(prefix, ...args);
}

export interface CallStackInfo {
  path: string;
  line: string;
  col: string;
  method: string;
  callStack: string[];
  inConsoleClass: boolean;
}

// https://v8.dev/docs/stack-trace-api
const callStackFmt = /at\s+(.*)\s+\((.*):(\d*):(\d*)\)/i;
const callStackFmt2 = /at\s+()(.*):(\d*):(\d*)/i;

export function getCallStackInfo(stackIndex = 0): CallStackInfo | null {
  if (disableStackCapture) return null;
  /*
   * Node.js implementation details:
   * ErrorCaptureStackTrace JS binding: https://github.com/nodejs/node/blob/e46c680bf2b211bbd52cf959ca17ee98c7f657f5/deps/v8/src/builtins/builtins-definitions.h#L509
   * v8::internal::ErrorCaptureStackTrace implementation: https://github.com/nodejs/node/blob/e46c680bf2b211bbd52cf959ca17ee98c7f657f5/deps/v8/src/builtins/builtins-error.cc#L27
   * v8::internal::Isolate::CaptureAndSetDetailedStackTrace: https://github.com/nodejs/node/blob/f37c26b8a2e10d0a53a60a2fad5b0133ad33308a/deps/v8/src/execution/isolate.cc#L1151
   * v8::internal::Isolate::CaptureSimpleStackTrace https://github.com/nodejs/node/blob/f37c26b8a2e10d0a53a60a2fad5b0133ad33308a/deps/v8/src/execution/isolate.cc#L1134
   */

  const callStack = new Error().stack?.split('\n') ?? [];
  const inConsoleClass =
    callStack[3]?.indexOf('internal/console') >= 0 ?? false;
  const offset = inConsoleClass ? 5 : 3;
  const userCallStack = callStack.slice(offset);
  const callInfo = userCallStack[stackIndex];
  const matches = callStackFmt.exec(callInfo) ?? callStackFmt2.exec(callInfo);
  if (!matches || matches.length !== 5) return null;
  const path = matches[2];
  return {
    path,
    line: matches[3],
    col: matches[4],
    method: matches[1],
    callStack: userCallStack,
    inConsoleClass,
  };
}
