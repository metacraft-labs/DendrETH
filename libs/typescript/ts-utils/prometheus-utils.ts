'use strict';
import { getGenericLogger } from './logger';

import client, { Histogram, Summary } from 'prom-client';
import express from 'express';

const register = new client.Registry();
const logger = getGenericLogger();

let followNetwork: string;

export function initPrometheusSetup(port?: number, curFollowNetwork?: string) {
  const app = express();
  prometheusInitProving();
  if (!port) {
    // Only for pollUpdates
    port = 2999;
  }
  if (curFollowNetwork) {
    followNetwork = curFollowNetwork;
  }

  // Only expose the metrics endpoint if not already initialized
  app.get('/metrics', async (req, res) => {
    res.set('Content-Type', register.contentType);
    res.end(await register.metrics());
  });

  app.listen(port, () => {
    console.log(`Prometheus metrics exposed on port ${port}`);
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

export function registerGaugesForProver() {
  register.registerMetric(timesGettingInputsForProofGeneration);
  register.registerMetric(numberOfProofGenerated);
}

export function registerGaugesForStartPublishing() {
  register.registerMetric(accountBalanceGauge);
  register.registerMetric(previousSlot);
  register.registerMetric(transactionForSlot);
  register.registerMetric(currentNetworkSlot);
  register.registerMetric(minutesDelayPrevSlot);
  register.registerMetric(minutesDelayTransaction);
  register.registerMetric(numberOfProofPublished);
}

export const accountBalanceGauge = new client.Gauge({
  name: 'account_balance',
  help: 'Current balance of the account',
  labelNames: ['network'],
});

export const previousSlot = new client.Gauge({
  name: 'previous_slot',
  help: 'Previous slot on the chain',
  labelNames: ['network'],
});

export const transactionForSlot = new client.Gauge({
  name: 'transaction_slot',
  help: 'Transaction publishing for slot',
  labelNames: ['network'],
});

export const currentNetworkSlot = new client.Gauge({
  name: 'current_network_slot',
  help: 'Current slot on chian',
  labelNames: ['network'],
});

export const minutesDelayPrevSlot = new client.Gauge({
  name: 'minutes_delay_prev_slot',
  help: 'How behind is the last slot',
  labelNames: ['network'],
});

export const minutesDelayTransaction = new client.Gauge({
  name: 'minutes_delay_transaction',
  help: 'How behind is the transaction',
  labelNames: ['network'],
});

export async function prometheusInitProving() {
  timesGettingInputsForProofGeneration.reset();
  numberOfProofGenerated.reset();
  numberOfProofPublished.reset();
}

export function incrementInputsForProofGeneration() {
  timesGettingInputsForProofGeneration.inc();
}

export function incrementProofGenerated() {
  numberOfProofGenerated.inc();
}

export function incrementProofPublished(network) {
  numberOfProofPublished.labels(network).inc();
}
export const timesGettingInputsForProofGeneration = new client.Counter({
  name: 'times_getting_inputs_for_proof_generation',
  help: 'The number of times inputs for proof generation were requested(since last restart)',
  labelNames: ['network'],
});

export const numberOfProofGenerated = new client.Counter({
  name: 'number_of_proof_generated',
  help: 'The number of proofs generated(since last restart)',
  labelNames: ['network'],
});

export const numberOfProofPublished = new client.Counter({
  name: 'number_of_proof_published',
  help: 'The number of proofs published(since last restart)',
  labelNames: ['network'],
});
