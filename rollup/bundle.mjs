import path from 'node:path';
import fs from 'node:fs/promises';
import { rollup } from 'rollup';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import peerDepsExternal from 'rollup-plugin-peer-deps-external';
import del from 'rollup-plugin-delete';
import { glob } from 'glob';
import { createRequire } from 'node:module';

main().catch(console.error);

function main() {
  if (process.argv.length !== 3) {
    console.log(`
    Usage:
        |yarn bundle PATH|

      Called with:
        |yarn bundle${process.argv.slice(2).join(' ')}|`);
    process.exit(1);
  }

  const packageDir = `${path.resolve(process.argv[2])}/`;

  // DO NOT REMOVE: Chesterson's fence
  process.chdir(packageDir);

  return Promise.all([
    build(packageDir, 'esm'), //
    build(packageDir, 'cjs'),
  ])
    .then(() =>
      fs.rename(`${packageDir}/dist/esm/types`, `${packageDir}/dist/types`),
    )
    .then(() => console.log('Build finished successfully'));
}

async function build(dir, format) {
  console.log(`Building ${dir} in ${format} format...`);

  let bundle;
  try {
    const config = createConfig(dir, format);
    bundle = await rollup(config);

    for (const outputOptions of config.output) {
      console.log(`Writing ${format}...`);
      await bundle.write(outputOptions);
      console.log(`Wrote ${format}`);
    }
  } catch (error) {
    console.error(error);
    process.exit(1);
  } finally {
    await bundle?.close();
  }
}

function createConfig(dir, format) {
  return {
    input: Object.fromEntries(
      glob
        .globSync(`${dir}/**/*.ts`)
        .filter(
          file =>
            !(
              file.endsWith('.test.ts') ||
              file.endsWith('.spec.ts') ||
              file.endsWith('.d.ts')
            ),
        )
        .map(file => {
          return [
            path.relative(
              dir,
              file.slice(0, file.length - path.extname(file).length),
            ),
            file,
          ];
        }),
    ),

    external: (id, parentId, isResolved) => {
      const absPath = path.isAbsolute(id)
        ? id
        : id.startsWith('.')
          ? path.resolve(parentId, id)
          : tryResolve(id, parentId);

      if (!absPath) {
        throw new Error(`Module ${id} was not resolved. Parent module: ${parentId}.`);
      }

      return !absPath.startsWith(dir);
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
        packageJsonPath: path.join(dir, 'package.json'),
      }),
      nodeResolve({
        extensions: ['.mjs', '.cjs', '.js', '.ts', '.json'],
        preferBuiltins: true,
        // This instructs Rollup to prioritize ESM over CJS when resolving modules
        mainFields: ['module', 'main'],
        modulesOnly: true,
      }),
      del({ targets: `${dir}/dist`, force: true }),
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

function tryResolve(id, parentId) {
  const cjsResolve = (id, parentId) => {
    const require = createRequire(parentId);
    return require.resolve(id);
  };

  for (const resolve of [import.meta.resolve, cjsResolve]) {
    try {
      return resolve(id, parentId);
    } catch (err) {
      if (process.env.DEBUG_PACKAGE_RESOLUTION?.toLowerCase() === 'yes') {
        console.log(
          '----------------------------------------------------------------------------------------------------------',
        );
        console.log(resolve.name);
        console.log({ id, parentId });
        console.log(err);
        console.log(
          '----------------------------------------------------------------------------------------------------------',
        );
      }
    }
  }

  return null;
}
