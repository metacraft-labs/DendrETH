import fs from 'fs-extra';

import { BVFileMock, RunConfig } from './balanceVerifierFileMock';
import { log } from './logging';
import { parseBoolEnvVar, setLogging, stringify } from './utils/common-utils';
import { resultsFile } from './utils/constants';
import { experimentalDir } from './utils/file-utils';

async function main() {
  const logWrites = parseBoolEnvVar('LOG_WRITES');
  const removeCompleteTasks = parseBoolEnvVar('REMOVE_COMPLETE_TASKS');

  const customConfig: RunConfig | null =
    process.env['CUSTOM_CONFIG'] !== 'true'
      ? null
      : {
          logWrites,
          removeCompleteTasks,
          depth: BigInt(process.env['DEPTH'] ?? '10'),
          validatorCount: BigInt(process.env['VALIDATOR_COUNT'] ?? '100'),
          sparseAmount: BigInt(process.env['SKIP'] ?? '3') + 1n,
        };

  await fs.rm(resultsFile, { force: true });

  const configs = customConfig ? [customConfig] : defaultConfigs;

  for (const config of configs) {
    const res = await benchmark({ logWrites, removeCompleteTasks, ...config });
    await fs.appendFile(resultsFile, stringify(res) + '\n');
  }
}

const defaultConfigs = [
  { depth: 4n, validatorCount: 2n ** 2n, sparseAmount: 1n },
  { depth: 4n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  { depth: 20n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  { depth: 20n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  { depth: 24n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  { depth: 37n, validatorCount: 2n ** 3n, sparseAmount: 3n },
  { depth: 37n, validatorCount: 2n ** 10n, sparseAmount: 3n },
  { depth: 37n, validatorCount: 2n ** 16n, sparseAmount: 3n },
  { depth: 37n, validatorCount: 2n ** 20n, sparseAmount: 3n },
  { depth: 37n, validatorCount: 2n ** 21n, sparseAmount: 3n },
];

type RunResult = {
  config: Omit<RunConfig, 'logWrites'>;
  runtimeMilliseconds: number;
};

export async function benchmark(config: RunConfig): Promise<RunResult> {
  log('config', config);

  setLogging(config.logWrites);

  const bVFileMock = new BVFileMock({
    ...config,
    directory: experimentalDir,
    shouldExist: x => x % config.sparseAmount === 0n,
  });

  const startTime = +new Date();

  await bVFileMock.setupWorkingDir();

  bVFileMock.prepareTasks();

  await bVFileMock.rootTask();

  const Δt = +new Date() - startTime;
  log('Task finished in ', `Δt: ${Δt} ms`);

  return { runtimeMilliseconds: Δt, config };
}

await main();
