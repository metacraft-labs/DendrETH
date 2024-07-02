'use strict';
import { getGenericLogger } from '@/ts-utils/logger';

import client, { Histogram, Summary } from 'prom-client';
import express from 'express';

const register = new client.Registry();
const logger = getGenericLogger();

let network: string;

export function initPrometheusSetup(port?: number, curNetwork?: string) {
  const app = express();

  if (!port) {
    // Only for pollUpdates
    port = 2999;
  }
  if (curNetwork) {
    network = curNetwork;
  }

  app.get('/metrics', async (req, res) => {
    res.end(await register.metrics());
  });

  app.listen(port, () => {
    console.log(`Express listening on port ${port}`);
  });

  return client;
}

export async function prometheusTiming<T>(func: () => T, funcName: string) {
  const functionExecutionTimeSummary = register.getSingleMetric(
    'function_execution_time_seconds',
  ) as Summary<string> | undefined;

  if (!functionExecutionTimeSummary) {
    const functionExecutionTimeSummary = new client.Summary({
      name: 'function_execution_time_seconds',
      help: 'Summary of function execution times',
      labelNames: ['function_name', 'network'],
      maxAgeSeconds: 3600,
    });
    register.registerMetric(functionExecutionTimeSummary);
  }

  const start = process.hrtime();
  try {
    const result = await func();
    return result;
  } finally {
    const end = process.hrtime(start);
    const durationSeconds = end[0] + end[1] / 1e9;
    logger.info(
      `Logging ${funcName} on ${network} - duration: ${durationSeconds}`,
    );

    if (functionExecutionTimeSummary) {
      functionExecutionTimeSummary
        .labels(funcName, network)
        .observe(durationSeconds);
    }
  }
}
