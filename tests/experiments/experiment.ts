import fs from 'fs-extra';
import { log, logError } from './logging';

const experimentalDir = 'tests/experiments/data';

type Tasks = Record<number, Promise<void>>;

const enablePrintOnRemove = false;
const enablePrintOnWrite = true;

export function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createArrayFromRange(x, y) {
  return Array.from({ length: y - x + 1 }, (_, index) => x + index);
}

export function stringify(obj: unknown) {
  return JSON.stringify(obj, (_, value) =>
    typeof value === 'bigint' ? value.toString() : value,
  );
}

export async function checkContent(gIndex: bigint) {
  const path = fileName(gIndex);
  const content = gIndex.toString();
  return (await fs.readFile(path, 'utf-8')).trim() == content;
}

export function childLeafsExists(gIndex: bigint) {
  const { leftChild, rightChild } = fromGI(gIndex);
  return Promise.all([checkContent(leftChild), checkContent(rightChild)]).then(
    ([a, b]) => a && b,
  );
}

export async function writeFile(gIndex: bigint, content: string) {
  if (enablePrintOnWrite) log('writing ', gIndex);
  await fs.writeFile(fileName(gIndex), content);
}

export async function removeFile(gIndex: bigint) {
  if (fs.existsSync(fileName(gIndex))) {
    await fs.rm(fileName(gIndex));
    if (enablePrintOnRemove) log('removed ', gIndex);
  }
}

export async function readFile(gIndex: bigint) {
  if (fs.existsSync(fileName(gIndex))) {
    return JSON.parse(await fs.readFile(fileName(gIndex), 'utf-8'));
  }
  return { status: 'not started', gIndex, data: 'no data' };
}

export async function execTask(
  gIndex: bigint,
  isLeaf: boolean,
  placeholder: boolean,
  delay = 0,
) {
  if (delay) await sleep(delay);
  const { leftChild, rightChild } = fromGI(gIndex);

  let content: {
    status: string;
    gIndex: bigint;
    data: string;
    left?: any;
    right?: any;
    isLeaf?: boolean;
  };

  if (placeholder) {
    content = { status: 'not started', gIndex, data: 'I am placeholder' };
  } else {
    if (isLeaf) {
      content = { status: 'done', gIndex, data: `I am leaf`, isLeaf };
    } else {
      const left = await readFile(leftChild);
      const right = await readFile(rightChild);

      if (left.data === 'no data' && right.data === 'no data') {
        content = {
          status: 'empty',
          gIndex,
          data: 'no data',
          left,
          right,
        };
      } else
        content = {
          status: 'done',
          gIndex,
          data: 'I am inner node',
          left,
          right,
        };
    }
  }

  if (isLeaf) {
    if (gIndex % 1000n === 0n) {
      await writeFile(gIndex, stringify(content));
    } else {
      log('skipping write', gIndex);
    }
  } else if (content.status == 'empty') {
    log('skipping Inner write ', gIndex);
  } else {
    await writeFile(gIndex, stringify(content));
  }
}

function log2(x: bigint) {
  let result = 0n;
  while (x > 1n) {
    x = x / 2n;
    result++;
  }
  return result;
}

export function iterateLevel(level: bigint): Generator<bigint, void, unknown> {
  const { levelBeg, levelEnd } = fromDepth(level);
  return range(levelBeg, levelEnd);
}

export function* range(
  begin: bigint,
  end: bigint,
): Generator<bigint, void, unknown> {
  for (let i = begin; i <= end; i++) {
    yield i;
  }
}

function fromGI(gIndex: bigint): { leftChild: bigint; rightChild: bigint } {
  return {
    leftChild: gIndex * 2n,
    rightChild: gIndex * 2n + 1n,
  };
}

function isLeaf(gIndex: bigint, depth: bigint): boolean {
  return gIndex >= fromDepth(depth).levelBeg;
}

function fromDepth(depth: bigint): {
  beg: bigint;
  end: bigint;
  levelBeg: bigint;
  levelEnd: bigint;
} {
  return {
    beg: 1n,
    end: 2n ** depth - 1n,
    levelBeg: 2n ** (depth - 1n),
    levelEnd: 2n ** depth - 1n,
  };
}

export function fileName(gIndex: bigint) {
  const level = log2(gIndex);
  return `${experimentalDir}/l${level}-gIndex${gIndex}.json`;
}

export async function writePlaceholderFiles(depth: bigint) {
  const { beg, end } = fromDepth(depth);
  return Promise.all(
    [...range(beg, end)].map(i => execTask(i, isLeaf(i, depth), true, 0)),
  );
}

export function executeTree(depth: bigint, tasks: Tasks, jobDelay = 0) {
  for (let level = depth; level >= 1; level--) {
    for (let gIndex of iterateLevel(level)) {
      if (level === depth) {
        tasks[`${gIndex}`] = execTask(gIndex, true, false, jobDelay);
        continue;
      }

      const { leftChild, rightChild } = fromGI(gIndex);
      tasks[`${gIndex}`] = Promise.all([
        tasks[`${leftChild}`],
        tasks[`${rightChild}`],
      ]).then(() => execTask(gIndex, false, false, jobDelay));
      // .then(() =>
      //   Promise.all([removeFile(leftChild), removeFile(rightChild)]),
      // );
    }
  }
}

export async function runIt() {
  const depth = BigInt(process.env['DEPTH'] ?? '10');

  fs.mkdir(experimentalDir, { recursive: true });

  // log('Writing placeholder files');
  // await writePlaceholderFiles(depth);
  // log('Finished writing placeholder files');

  const tasks: Tasks = {};
  executeTree(depth, tasks);

  await tasks[1];
  log('working here finished');
}

runIt()
  .then(() => {
    log('done');
    process.exit(0);
  })
  .catch(e => {
    logError(e);
    process.exit(1);
  });
