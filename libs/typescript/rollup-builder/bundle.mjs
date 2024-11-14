import { loadConfigFile } from 'rollup/loadConfigFile';
import path from 'node:path';
import { rollup } from 'rollup';

if (process.argv.length != 3) {
	process.exit(1);
}

const workspaceDir = process.argv[2];

// load the config file next to the current script;
// the provided config object has the same effect as passing "--format es"
// on the command line and will override the format of all outputs
const configFilepath = path.resolve(import.meta.dirname, 'rollup.config.mjs');
console.log({configFilepath});
console.log({ cwd: process.cwd()});
const { options, warnings } = await loadConfigFile(configFilepath, { format: 'es' });;

// "warnings" wraps the default `onwarn` handler passed by the CLI.
// This prints all warnings up to this point:
console.log(`We currently have ${warnings.count} warnings`);

// This prints all deferred warnings
warnings.flush();

// options is an array of "inputOptions" objects with an additional
// "output" property that contains an array of "outputOptions".
// The following will generate all outputs for all inputs, and write
// them to disk the same way the CLI does it:
for (const optionsObj of options) {
	const bundle = await rollup(optionsObj);
	await Promise.all(optionsObj.output.map(bundle.write));
}
