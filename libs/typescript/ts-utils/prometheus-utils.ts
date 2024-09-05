'use strict';
import { getGenericLogger } from './logger';

import client, { Histogram, Summary } from 'prom-client';
import express from 'express';

const register = new client.Registry();
const logger = getGenericLogger();

let followNetwork: string;

export function initPrometheusSetup(port?: number, curFollowNetwork?: string) {
  const app = express();

  if (!port) {
    // Only for pollUpdates
    port = 2999;
  }
  if (curFollowNetwork) {
    followNetwork = curFollowNetwork;
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
    if (followNetwork == undefined) {
      logger.info(
        `Executing method: ${funcName} with duration: ${durationSeconds}`,
      );
    } else {
      logger.info(
        `Executing method: ${funcName} on follow-network: ${followNetwork} with duration: ${durationSeconds}`,
      );
    }
    if (functionExecutionTimeSummary) {
      functionExecutionTimeSummary
        .labels(funcName, followNetwork)
        .observe(durationSeconds);
    }
  }
}
