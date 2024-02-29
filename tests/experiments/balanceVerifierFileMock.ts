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
  childrenFromGIndex,
  iterateLevel,
  TreeParams,
  iterateTree,
  gIndexToLevel,
  isLeaf,
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
import { BalanceVerifier } from './interfaces';

type BVFileConfig = {
  logger: boolean;
  directory: string;
  depth: bigint;
  validatorCount: bigint;
  sparseAmount: bigint;
  shouldExist: (gIndex: bigint) => boolean;
};

class BVFileMock implements BalanceVerifier {
  configuration: BVFileConfig;
  tasks: Tasks = {};

  constructor(configuration: BVFileConfig) {
    this.configuration = configuration;
    this.tasks = {};

    setLogging(configuration.logger);
  }

  async setupWorkingDir() {
    await fs.rm(this.configuration.directory, { recursive: true, force: true });
    await fs.mkdir(this.configuration.directory, { recursive: true });
  }

  async execTask(gIndex: bigint, delay = 0) {
    const isLeaff = isLeaf(gIndex, this.configuration.depth);
    if (delay) await sleep(delay);
    const { leftChild, rightChild } = childrenFromGIndex(gIndex);

    let nodeData: NodeData;
    if (isLeaff) {
      nodeData = {
        gIndex,
        content: {
          gIndex,
          hash: sha256(stringToBytes(JSON.stringify(exampleLeafData))),
          data: exampleLeafData,
        },
        isLeaf: isLeaff,
      };
    } else {
      const left = await readFile(leftChild);
      const right = await readFile(rightChild);

      if (left.isMissing && right.isMissing && true) {
        nodeData = {
          gIndex,
          content: { gIndex, hash: zeroHashes[`${gIndexToLevel(gIndex)}`] }, // TODO: Use pre-calculated hash for this depth
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

    if (isLeaff) {
      if (this.configuration.shouldExist(gIndex)) {
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

  prepareTasks(jobDelay = 0) {
    // const { depth, validatorCount: lastValidatorIndex, shouldExist } = treeParams;
    for (let { indexOnThisLevel, gIndex, level } of iterateTree(
      this.configuration.depth,
      this.configuration.validatorCount,
    )) {
      if (level === this.configuration.depth) {
        this.tasks[`${gIndex}`] = this.execTask(gIndex, jobDelay);
        continue;
      }

      const { leftChild, rightChild } = childrenFromGIndex(gIndex);
      this.tasks[`${gIndex}`] = Promise.all([
        this.tasks[`${leftChild}`],
        this.tasks[`${rightChild}`],
      ]).then(() => this.execTask(gIndex, jobDelay));
      // .then(() => Promise.all([removeFile(leftChild), removeFile(rightChild)]));
    }
  }

  async executeTasks() {
    await this.tasks[1];
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

  const startTime = new Date().getTime();

  log('config', { logWrites, depth, validatorCount, sparseAmount });

  const bVFileMock = new BVFileMock({
    logger: logWrites,
    directory: experimentalDir,
    depth,
    validatorCount,
    sparseAmount,
    shouldExist: x => x % sparseAmount === 0n,
  });

  await bVFileMock.setupWorkingDir();

  bVFileMock.prepareTasks();

  await bVFileMock.tasks[1];

  const now = new Date();
  const diff = `${(now.getTime() - startTime).toString(10)}`.padStart(2);
  log('Task finished in ', `Δt₀: ${diff} ms`);

  const results = {
    config: `{ depth: ${depth}, validatorCount: ${validatorCount}, sparseAmount: ${sparseAmount} }`,
    time: `Δt₀: ${diff} ms`,
  };

  return results;
}

const configs = [
  { depth: 4n, validatorCount: 2n ** 2n, sparseAmount: 1n },
  // { depth: 4n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  // { depth: 20n, validatorCount: 2n ** 3n, sparseAmount: 3n },
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
