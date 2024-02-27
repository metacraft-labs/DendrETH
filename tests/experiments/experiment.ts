import fs from 'fs-extra';
import { log, logError } from './logging';
import {
  readFile,
  writeFile,
  experimentalDir,
  removeFile,
} from './utils/file-utils';
import {
  NodeData,
  fromGIndex,
  iterateLevel,
  TreeParams,
} from './utils/tree-utils';
import { exampleLeafData } from './utils/constants';
import { Tasks, sleep, stringToBytes, stringify } from './utils/common-utils';

import { sha256 } from 'ethers/lib/utils';

// import debug from 'debug';
// debug.enable('experimets:*');

// const log = debug('experimets:*');

export async function execTask(
  gIndex: bigint,
  isLeaf: boolean,
  placeholder: boolean,
  delay = 0,
  shouldExist: (gIndex: bigint) => boolean,
) {
  if (delay) await sleep(delay);
  const { leftChild, rightChild } = fromGIndex(gIndex);

  let nodeData: NodeData;
  if (placeholder) {
    nodeData = {
      gIndex,
      content: { gIndex, hash: '' },
      isPlaceholder: true,
    };
  } else {
    if (isLeaf) {
      nodeData = {
        gIndex,
        content: {
          gIndex,
          hash: sha256(stringToBytes(JSON.stringify(exampleLeafData))),
          data: exampleLeafData,
        },
        isLeaf,
      };
    } else {
      const left = await readFile(leftChild);
      const right = await readFile(rightChild);

      if (left.isMissing && right.isMissing && true) {
        nodeData = {
          gIndex,
          content: { gIndex, hash: '' }, // TODO: Use pre-calculated hash for this depth
          isMissing: true,
        };
      } else
        nodeData = {
          gIndex,
          content: {
            gIndex,
            hash: sha256(stringToBytes(left.content.hash + right.content.hash)),
          },
          isMissing: false,
        };
    }
  }

  if (isLeaf) {
    if (shouldExist(gIndex)) {
      await writeFile(gIndex, stringify(nodeData.content));
    } else {
      log('skipping write', gIndex);
    }
  } else if (nodeData.isMissing) {
    log('skipping Inner write ', gIndex);
  } else {
    await writeFile(gIndex, stringify(nodeData.content));
  }
}

export function executeTree(
  treeParams: TreeParams,
  tasks: Tasks,
  jobDelay = 0,
) {
  const { depth, validatorCount: lastValidatorIndex, shouldExist } = treeParams;
  for (let level = depth; level >= 1; level--) {
    for (let { indexOnThisLevel, gIndex } of iterateLevel(level)) {
      if (level === depth) {
        if (indexOnThisLevel > lastValidatorIndex) break;
        tasks[`${gIndex}`] = execTask(
          gIndex,
          true,
          false,
          jobDelay,
          shouldExist,
        );
        continue;
      }

      const { leftChild, rightChild } = fromGIndex(gIndex);
      tasks[`${gIndex}`] = Promise.all([
        tasks[`${leftChild}`],
        tasks[`${rightChild}`],
      ])
        .then(() => execTask(gIndex, false, false, jobDelay, shouldExist))
        .then(() =>
          Promise.all([removeFile(leftChild), removeFile(rightChild)]),
        );
    }
  }
}

export async function runIt() {
  const depth = BigInt(process.env['DEPTH'] ?? '10'),
    validatorCount = BigInt(process.env['VALIDATOR_COUNT'] ?? '100'),
    sparseAmount = BigInt(process.env['SKIP'] ?? '3') + 1n;

  console.log({ depth, validatorCount, sparseAmount });

  const treeParams: TreeParams = {
    depth,
    validatorCount,
    shouldExist: x => x % sparseAmount === 0n,
  };

  fs.mkdir(experimentalDir, { recursive: true });

  // log('Writing placeholder files');
  // await writePlaceholderFiles(depth);
  // log('Finished writing placeholder files');

  const tasks: Tasks = {};
  executeTree(treeParams, tasks);

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
