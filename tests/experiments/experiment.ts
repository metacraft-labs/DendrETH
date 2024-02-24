import fs from 'fs-extra';
import { loopWhile } from '../../libs/typescript/ts-utils/common-utils';

const experimentalDir = 'tests/experiments/data';

type Tasks = Record<number, Promise<void>>;

export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createArrayFromRange(x, y) {
  return Array.from({ length: y - x + 1 }, (_, index) => x + index);
}

export async function checkContent(path: string, content: string) {
  return (await fs.readFile(path, 'utf-8')).trim() == content;
}

export async function childLeafsExists(
  dataDir: string,
  childLevel,
  gIndex: number,
) {
  const childGIndexL = Math.floor(gIndex * 2);
  const childGIndexR = Math.floor(gIndex * 2) + 1;
  const childGIndexLPath = `${dataDir}/l${childLevel}-gindex${childGIndexL}.txt`;
  const childGIndexRPath = `${dataDir}/l${childLevel}-gindex${childGIndexR}.txt`;
  return (
    (await checkContent(childGIndexLPath, `${childGIndexL}`)) &&
    (await checkContent(childGIndexRPath, `${childGIndexR}`))
  );
}

export async function prepareTask(gindex, level: number, delay = 0) {
  if (delay) {
    await sleep(delay);
  }
  await fs.writeFile(
    `${experimentalDir}/l${level}-gindex${gindex}.txt`,
    `${gindex}`,
  );
  console.log('wrote ', gindex);
}

export function prepareTasks(depth: number, tasks: Tasks) {
  // const depth = Math.log2(allElements) + 1;
  const firsGIndexOnLastLevel = Math.pow(2, depth - 1);
  const finalGIndexOnLastLevel = Math.pow(2, depth) - 1;
  for (let i = finalGIndexOnLastLevel; i >= firsGIndexOnLastLevel; i--) {
    tasks[i] = prepareTask(i, depth);
  }
  for (let level = depth - 1; level >= 1; level--) {
    const startIndex = Math.pow(2, level - 1);
    const finalIndex = Math.pow(2, level) - 1;
    for (let gIndex = startIndex; gIndex <= finalIndex; gIndex++) {
      const childGIndexL = Math.floor(gIndex * 2);
      const childGIndexR = Math.floor(gIndex * 2) + 1;
      tasks[gIndex] = Promise.all([tasks[childGIndexL], tasks[childGIndexR]])
        .then(() => prepareTask(gIndex, level))
        .then(() => {
          fs.rm(`${experimentalDir}/l${level + 1}-gindex${childGIndexL}.txt`);
          fs.rm(`${experimentalDir}/l${level + 1}-gindex${childGIndexR}.txt`);
          console.log('removed ', childGIndexL, childGIndexR);
        });
    }
  }
}

export async function runIt() {
  const tasks = {} as Tasks;

  const initElements = 20;

  fs.mkdir(experimentalDir, { recursive: true });

  prepareTasks(initElements, tasks);

  await tasks[1];
  console.log('working here finished');
}

runIt()
  .then(() => {
    console.log('done');
    process.exit(0);
  })
  .catch(e => {
    console.error(e);
    process.exit(1);
  });
