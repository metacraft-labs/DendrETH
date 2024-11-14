import { glob } from 'glob';
import path from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'node:module';

import typescript from 'rollup-plugin-typescript2';
import peerDepsExternal from 'rollup-plugin-peer-deps-external';
import del from 'rollup-plugin-delete';
import json from '@rollup/plugin-json';
import commonjs from '@rollup/plugin-commonjs';
import { nodeResolve } from '@rollup/plugin-node-resolve';

export function getDefault(fileUrl) {
  const dir = path.dirname(fileURLToPath(fileUrl));

  const require = createRequire(import.meta.url);

  const fileExtensions = ['.mjs', '.cjs', '.js', '.ts', '.json'];

  const plugins = [
    json(),
    commonjs(),

    peerDepsExternal({
      packageJsonPath: path.join(dir, 'package.json'),
    }),
    nodeResolve({
      extensions: fileExtensions,
      preferBuiltins: true,
      mainFields: ['module', 'main'], // This instructs Rollup to prioritize ESM over CJS when resolving modules
      modulesOnly: true,
    }),
    typescript({
      check: true,
      tsconfig: path.join(dir, 'tsconfig.json'),
      useTsconfigDeclarationDir: true,
    }),
    del({ targets: `${dir}/dist/*` }),
  ];

  const config = {
    input: Object.fromEntries(
      glob
        .globSync(`${dir}/src/**/*.ts`)
        .filter(file => {
          if (file.endsWith('spec.ts')) {
            return false;
          }
          return true;
        })
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
            fileURLToPath(new URL(file, import.meta.url)),
          ];
        }),
    ),
    external: (id, parentId, isResolved) => {
      const check = id.startsWith('@effect') || !id.includes(`${dir}/src`);
      const absPath = path.isAbsolute(id)
        ? id
        : id.startsWith('.')
          ? path.resolve(parentId, id)
          : require.resolve(id);

      const internal = absPath.startsWith(`${dir}/src`);

      return !internal;
    },

    output: [
      {
        dir: `${dir}/dist/esm`,
        format: 'es',
        entryFileNames: '[name].mjs',
      },
      {
        dir: `${dir}/dist/cjs`,
        format: 'cjs',
        entryFileNames: '[name].cjs',
      },
    ],
    plugins,
  };

  return config;
}

export default getDefault(import.meta.url);

