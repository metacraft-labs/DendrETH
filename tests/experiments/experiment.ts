import fs from 'fs-extra';
import { log, logError } from './logging';
import { readFile, writeFile, experimentalDir } from './utils/file-utils';
import {
  NodeData,
  TreeParams,
  iterateTree,
  gIndexToDepth,
} from './utils/tree-utils';
import { exampleLeafData, resultsFile, zeroHashes } from './utils/constants';
import {
  Tasks,
  logWrite,
  setLogging,
  sleep,
  stringToBytes,
  stringify,
} from './utils/common-utils';

import { sha256 } from 'ethers/lib/utils';
import { fromGIndex } from './utils/gindex';

export async function execTask(
  gIndex: bigint,
  isLeaf: boolean,
  placeholder: boolean,
  delay = 0,
  shouldExist: (gIndex: bigint) => boolean,
) {
  if (delay) await sleep(delay);

  const gIndexData = fromGIndex(gIndex);
  const leftChild = gIndexData.left;
  const rightChild = gIndexData.right;

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
          content: { gIndex, hash: zeroHashes[`${gIndexToDepth(gIndex)}`] }, // TODO: Use pre-calculated hash for this depth
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
      logWrite(gIndex, 'skipping write');
    }
  } else if (nodeData.isMissing) {
    logWrite(gIndex, 'skipping Inner write');
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
  for (let { indexOnThisLevel, gIndex, level } of iterateTree({
    depth,
    lastLeafIndex: lastValidatorIndex,
  })) {
    if (level === depth) {
      tasks[`${gIndex}`] = execTask(gIndex, true, false, jobDelay, shouldExist);
      continue;
    }

    const gIndexData = fromGIndex(gIndex);
    const leftChild = gIndexData.left;
    const rightChild = gIndexData.right;

    tasks[`${gIndex}`] = Promise.all([
      tasks[`${leftChild}`],
      tasks[`${rightChild}`],
    ]).then(() => execTask(gIndex, false, false, jobDelay, shouldExist));
    // .then(() => Promise.all([removeFile(leftChild), removeFile(rightChild)]));
  }
}

export async function runIt(config?) {
  const logWrites = (process.env['LOG_WRITES'] ?? 'true') === 'true';
  const { depth, validatorCount, sparseAmount } = config
    ? config
    : {
        depth: BigInt(process.env['DEPTH'] ?? '10'),
        validatorCount: BigInt(process.env['VALIDATOR_COUNT'] ?? '100'),
        sparseAmount: BigInt(process.env['SKIP'] ?? '3') + 1n,
      };

  setLogging(logWrites);
  const startTime = new Date().getTime();

  log('config', { logWrites, depth, validatorCount, sparseAmount });

  const treeParams: TreeParams = {
    depth,
    validatorCount,
    shouldExist: x => x % sparseAmount === 0n,
  };

  await fs.rm(experimentalDir, { recursive: true, force: true });
  await fs.mkdir(experimentalDir, { recursive: true });

  const tasks: Tasks = {};
  executeTree(treeParams, tasks);

  await tasks[1];
  const now = new Date();
  const diff = `${(now.getTime() - startTime).toString(10)}`.padStart(2);
  log('  ', `Δt₀: ${diff} ms`);

  const results = {
    config: `{ depth: ${depth}, validatorCount: ${validatorCount}, sparseAmount: ${sparseAmount} }`,
    time: `Δt₀: ${diff} ms`,
  };

  return results;
}

const configs = [
  { depth: 4n, validatorCount: 2n ** 2n, sparseAmount: 1n },
  { depth: 4n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  { depth: 20n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  // { depth: 20n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  // { depth: 24n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  // { depth: 37n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  // { depth: 37n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  // { depth: 37n, validatorCount: 2n ** 16n, sparseAmount: 3n },
  // { depth: 37n, validatorCount: 2n ** 20n, sparseAmount: 3n },
  // { depth: 37n, validatorCount: 2n ** 21n, sparseAmount: 3n },
];

const executeTasks = async () => {
  await fs.rm(resultsFile, { force: true });
  let results: [{ config: string; time: string }] = [
    { config: 'config', time: 'time' },
  ];

  for (const config of configs) {
    results.push(await runIt(config));
  }
  await fs.writeFile(resultsFile, JSON.stringify(results));
};

executeTasks()
  .then(() => {
    log('Done');
    process.exit(0);
  })
  .catch(e => {
    logError(e);
    process.exit(1);
  });
