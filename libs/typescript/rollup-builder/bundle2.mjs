import path from 'node:path';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

import { rollup } from 'rollup';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import { nodeResolve } from '@rollup/plugin-node-resolve';
// import typescript from 'rollup-plugin-typescript2';
import typescript from '@rollup/plugin-typescript';
import peerDepsExternal from 'rollup-plugin-peer-deps-external';
import del from 'rollup-plugin-delete';

import { glob } from 'glob';

if (process.argv.length !== 3) {
  console.log(`
    Usage:
        |yarn dendreth-bundle PATH|

      Called with:
        |yarn dendreth-bundle${process.argv.slice(2).join(' ')}|`);
  process.exit(1);
}

const dir = path.resolve(process.argv[2]);
console.log({ dir });

await Promise.all([
  build(dir, 'esm'), //
  build(dir, 'cjs'),
])
  .then(() => fs.rename(`${dir}/dist/esm/types`, `${dir}/dist/types`))
  .then(() => console.log('Build finished successfully'));

async function build(dir, format) {
  let bundle;
  try {
    const config = createConfig(dir, format);

    console.log(`rollup(${format})...`);
    bundle = await rollup(config);
    console.log(`rollup(${format}) done.`);

    for (const outputOptions of config.output) {
      const { output } = await bundle.write(outputOptions);
    }
  } catch (error) {
    console.error(error);
    process.exit(1);
  } finally {
    await bundle?.close();
  }
}

function createConfig(dir, format) {
  const require = createRequire(import.meta.url);
  return {
    input: Object.fromEntries(
      glob
        .globSync(`${dir}/src/**/*.ts`)
        .filter(file => !file.endsWith('spec.ts'))
        .map(file => {
          return [
            // This remove `src/` as well as the file extension from each
            // file, so e.g. src/nested/foo.js becomes nested/foo
            path.relative(
              `${dir}/src`,
              file.slice(0, file.length - path.extname(file).length),
            ),
            // This expands the relative paths to absolute paths, so e.g.
            // src/nested/foo becomes /project/src/nested/foo.js
            fileURLToPath(new URL(file, `file://${dir}`)),
            //path.resolve(dir, file),
          ];
        }),
    ),

    external: (id, parentId, isResolved) => {
      const absPath = path.isAbsolute(id)
        ? id
        : id.startsWith('.')
        ? path.resolve(parentId, id)
        : require.resolve(id);

      const internal = absPath.startsWith(dir);

      return !internal;
    },
    output: [
      {
        dir: `${dir}/dist/${format}`,
        format: format,
        entryFileNames: `[name].${format}`,
      },
    ],
    plugins: [
      json(),
      commonjs(),
      peerDepsExternal({
        packageJsonPath: `${dir}/package.json`,
      }),
      nodeResolve({
        extensions: ['.mjs', '.cjs', '.js', '.ts', '.json'],
        preferBuiltins: true,
        // This instructs Rollup to prioritize ESM over CJS when resolving modules
        mainFields: ['module', 'main'],
        modulesOnly: true,
      }),
      del({ targets: `${dir}/dist` }),
      typescript({
        tsconfig: `${dir}/tsconfig.json`,
        composite: false,
        declaration: format != 'cjs',
        declarationMap: format != 'cjs',
        noEmit: format == 'cjs',
        outDir: format == 'cjs' ? undefined : `${dir}/dist/${format}/types`,
        declarationDir:
          format == 'cjs' ? undefined : `${dir}/dist/${format}/types`,
      }),
    ],
  };
}
