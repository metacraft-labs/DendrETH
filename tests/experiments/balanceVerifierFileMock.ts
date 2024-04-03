import fs from 'fs-extra';
import assert from 'assert';
import { sha256 } from 'ethers/lib/utils';

import { readFile, writeFile, removeFile } from './utils/file-utils';
import {
  NodeData,
  iterateTree,
  gIndexToDepth,
  isLeaf,
} from './utils/tree-utils';
import { exampleLeafData, zeroHashes } from './utils/constants';
import {
  Tasks,
  logWrite,
  sleep,
  stringToBytes,
  stringify,
} from './utils/common-utils';

import { BalanceVerifier } from './interfaces';
import { fromGIndex } from './utils/gindex';

export type RunConfig = {
  logWrites: boolean;
  removeCompleteTasks: boolean;
  depth: bigint;
  validatorCount: bigint;
  sparseAmount: bigint;
};

export type BVFileConfig = RunConfig & {
  directory: string;
  shouldExist: (gIndex: bigint) => boolean;
};

export class BVFileMock implements BalanceVerifier {
  readonly configuration: BVFileConfig;
  private readonly tasks: Tasks = {};

  constructor(configuration: BVFileConfig) {
    this.configuration = configuration;
    this.tasks = {};
  }

  async setupWorkingDir() {
    await fs.rm(this.configuration.directory, { recursive: true, force: true });
    await fs.mkdir(this.configuration.directory, { recursive: true });
  }

  async execTask(gIndex: bigint, delay = 0) {
    const isLeaff = isLeaf(gIndex, this.configuration.depth);
    if (delay) await sleep(delay);

    const gIndexData = fromGIndex(gIndex);
    const leftChild = gIndexData.left;
    const rightChild = gIndexData.right;

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

    if (this.configuration.removeCompleteTasks) {
      await Promise.all([removeFile(leftChild), removeFile(rightChild)]);
    }
  }

  prepareTasks(jobDelay = 0) {
    const { depth, validatorCount: lastLeafIndex } = this.configuration;
    for (let { gIndex, level } of iterateTree({ depth, lastLeafIndex })) {
      if (level === this.configuration.depth) {
        this.tasks[`${gIndex}`] = this.execTask(gIndex, jobDelay);
        continue;
      }

      const gIndexData = fromGIndex(gIndex);
      const leftChild = gIndexData.left;
      const rightChild = gIndexData.right;

      this.tasks[`${gIndex}`] = Promise.all([
        this.tasks[`${leftChild}`],
        this.tasks[`${rightChild}`],
      ]).then(() => this.execTask(gIndex, jobDelay));
    }
  }

  rootTask() {
    assert(this.tasks[1], 'root task not found');
    return this.tasks[1];
  }
}
