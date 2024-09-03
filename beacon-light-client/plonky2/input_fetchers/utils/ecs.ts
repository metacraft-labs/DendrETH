// Expects the following environment variables:
//   - ECS_REGION
//   - ECS_CLUSTER
//   - ECS_TASKDEF
//   - ECS_CONTAINER
//   - ECS_SUBNETS

import {
  DescribeTasksCommand,
  DescribeTasksCommandOutput,
  DesiredStatus,
  ECSClient,
  RunTaskCommand,
  RunTaskCommandInput,
  RunTaskCommandOutput,
  Task,
  TaskStopCode,
} from '@aws-sdk/client-ecs';

// +------+
// | Misc |
// +------+

function log(s: string): void {
  const now: string = new Date().toISOString();
  console.log(now, s);
}

function err(s: string): void {
  const now: string = new Date().toISOString();
  console.error(now, s);
}

async function sleep(ms: number): Promise<void> {
  async function executor(
    resolve: (_: void) => void,
    _reject: (_: void) => void,
  ): Promise<void> {
    setTimeout(resolve, ms);
  }
  return new Promise<void>(executor);
}

export async function retry<T>(f: () => PromiseLike<T>): Promise<T> {
  let lastError: unknown = '';
  for (let i = 1; i <= 3; i++) {
    try {
      return await f();
    } catch (e: unknown) {
      lastError = e;
      err(`[W] retry: Call ${i}/3 failed, retrying...`);
      await sleep(i * 1_000);
    }
  }
  err('[W] retry: All retry attempts failed');
  throw lastError;
}

// +-------+
// | Tasks |
// +-------+

class Environment {
  region: string;
  cluster: string;
  taskdef: string;
  container: string;
  subnets: string[];

  constructor() {
    this.region = '';
    this.cluster = '';
    this.taskdef = '';
    this.container = '';
    this.subnets = [];

    // Read a single environment variable.
    function get(name: string): string {
      const full: string = `ECS_${name}`;
      if (!process.env[full]) {
        err(`[E] environment variable ${full} is not set`);
        process.exit(1);
      }
      return '' + process.env[full];
    }

    ['REGION', 'CLUSTER', 'TASKDEF', 'CONTAINER'].forEach((name: string) => {
      const key: string = name.toLocaleLowerCase();
      const value: string = get(name);
      this[key] = value;
    });

    const subnets: string = get('SUBNETS');
    this.subnets = subnets.split(',');
  }
}

const ENV: Environment = new Environment();

function makeClient(): ECSClient {
  return new ECSClient({ region: ENV.region });
}

function extractArns(tasks: Task[]): string[] {
  const arns: string[] = tasks.map((task: Task): string => {
    if (task.taskArn == null || task.taskArn.length <= 0) {
      throw new Error('TODO');
    }
    return task.taskArn;
  });
  return arns;
}

// Re-fetch tasks by their ARNs.
async function refreshTasksByArns(
  ecsClient: ECSClient,
  arns: string[],
): Promise<Task[]> {
  if (arns.length <= 0) {
    return [];
  }
  const resp: DescribeTasksCommandOutput = await ecsClient.send(
    new DescribeTasksCommand({
      cluster: ENV.cluster,
      tasks: arns,
    }),
  );
  if (resp.tasks == null || resp.tasks.length != arns.length) {
    throw new Error('TODO');
  }
  return resp.tasks;
}

// Given a list of tasks, re-fetch their representation from ECS.
async function refreshTasks(
  ecsClient: ECSClient,
  tasks: Task[],
): Promise<Task[]> {
  const arns: string[] = extractArns(tasks);
  return refreshTasksByArns(ecsClient, arns);
}

// Return true if all tasks are stopped.
function allStopped(tasks: Task[]): boolean {
  for (let i = 0; i < tasks.length; i++) {
    if (tasks[i].lastStatus !== DesiredStatus.STOPPED) {
      return false;
    }
  }
  return true;
}

// Return true if all tasks are stopped completed successfully.
function countSuccessful(tasks: Task[]): number {
  let ans: number = 0;
  for (let i = 0; i < tasks.length; i++) {
    const x: Task = tasks[i];
    if (
      x.lastStatus === DesiredStatus.STOPPED &&
      x.stopCode === TaskStopCode.ESSENTIAL_CONTAINER_EXITED
    ) {
      //
      ans += 1;
    }
  }
  return ans;
}

// +------+
// | Main |
// +------+

// Run the task with `count` many instances, return the number of
// successfully completed tasks.
export default async function runTask(count: number): Promise<number> {
  const BATCH: number = 10; // 10 tasks at most per request.

  const ecsClient: ECSClient = makeClient();
  let tasks: Task[] = [];

  // A single request can start up to 10 tasks.
  for (let left = count; left > 0; left -= BATCH) {
    let batch = Math.min(left, BATCH);

    const params: RunTaskCommandInput = {
      cluster: ENV.cluster,
      capacityProviderStrategy: [
        {
          capacityProvider: 'FARGATE',
          weight: 1,
          base: 0,
        },
      ],
      taskDefinition: ENV.taskdef,
      count: batch,
      networkConfiguration: {
        awsvpcConfiguration: {
          subnets: ENV.subnets,
          assignPublicIp: 'ENABLED',
        },
      },
      overrides: {
        containerOverrides: [
          {
            name: ENV.container,
          },
        ],
      },
    };

    // Run ECS tasks.
    let data: RunTaskCommandOutput;
    try {
      data = await retry(() => ecsClient.send(new RunTaskCommand(params)));
    } catch (e: unknown) {
      throw e;
    }

    if (data.tasks != null) {
      tasks = tasks.concat(data.tasks);
    }
  }

  // Wait for tasks to complete.
  while (1) {
    log(`[I] runTask: checking ${tasks.length} instances...`);

    const snapshot: Task[] = await retry(() => refreshTasks(ecsClient, tasks));
    const stopped: boolean = allStopped(snapshot);

    if (stopped) {
      return countSuccessful(snapshot);
    }

    await sleep(30_000);
  }

  throw new Error('unreachable');
}
