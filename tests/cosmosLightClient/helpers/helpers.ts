import { promisify } from 'node:util';
import { exec as exec_ } from 'node:child_process';

const exec = promisify(exec_);

export function replaceInTextProof(updateFile) {
  let t = 0;
  const result = updateFile.replace(/proof/g, match =>
    ++t === 1 ? 'update' : match,
  );
  return result;
}

export class gasUsed {
  description: string;
  gas: number;

  constructor(description: string, gas: number) {
    this.description = description;
    this.gas = gas;
  }
}
