import * as fs from 'fs';
import * as path from 'path';

export interface Proof {
  a: string[];
  b: string[][];
  c: string[];
}

function isFileMatchingPattern(
  filePattern: string | undefined,
  filename: string,
) {
  if (!filePattern) {
    return true;
  }
  const regex = new RegExp(`^${filePattern.replace(/\*/g, '.*')}$`);
  return regex.test(filename);
}

export function getFilesInDir(_path: string, pattern?: string) {
  let files: Buffer[] = [];
  const content = fs.readdirSync(_path, {
    encoding: 'utf-8',
    withFileTypes: true,
  });
  for (let f of content) {
    if (f.isDirectory()) {
      files = [...files, ...getFilesInDir(path.join(_path, f.name), pattern)];
    } else if (isFileMatchingPattern(pattern, f.name)) {
      files.push(fs.readFileSync(path.join(_path, f.name)));
    }
  }
  return files;
}
