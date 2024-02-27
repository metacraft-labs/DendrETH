import fs from 'fs-extra';
import { log } from '../logging';
import { execTask } from '../experiment';
import { fromGI, fromDepth, range, isLeaf, log2, NodeData } from './tree-utils';

export const experimentalDir = 'tests/experiments/data';

export const enablePrintOnRemove = false;
export const enablePrintOnWrite = true;

export function fileName(gIndex: bigint) {
  const level = log2(gIndex);
  return `${experimentalDir}/l${level}-gIndex${gIndex}.json`;
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

export async function readFile(gIndex: bigint): Promise<NodeData> {
  if (fs.existsSync(fileName(gIndex))) {
    // return JSON.parse(await fs.readFile(fileName(gIndex), 'utf-8'));
    const NodeContent = JSON.parse(
      await fs.readFile(fileName(gIndex), 'utf-8'),
    );
    return { gIndex, content: NodeContent, isMissing: false };
  }
  // return { status: 'not started', gIndex, data: 'no data' };
  return { gIndex, content: { gIndex, hash: '' }, isMissing: true };
}

export async function writePlaceholderFiles(depth: bigint) {
  const { beg, end } = fromDepth(depth);
  return Promise.all(
    [...range(beg, end)].map(i =>
      execTask(i.gIndex, isLeaf(i.gIndex, depth), true, 0, () => true),
    ),
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
