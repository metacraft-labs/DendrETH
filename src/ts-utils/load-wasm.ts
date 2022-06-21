import { readFile } from 'fs/promises';

export async function loadWasm<Exports extends WebAssembly.Exports>({
  from,
  importObject,
}: {
  from: { url: URL } | { filepath: string };
  importObject?: WebAssembly.Imports;
}) {
  let res: WebAssembly.WebAssemblyInstantiatedSource;
  if ('url' in from && from.url instanceof URL) {
    const url = from.url;
    const resp = fetch(url);
    res = await WebAssembly.instantiateStreaming(resp, importObject);
  } else if ('filepath' in from && typeof from.filepath === 'string') {
    // TODO: use fetch + WebAssembly.instantiateStreaming when it's supported
    // to merge this with the above implementation
    const url = new URL(from.filepath, import.meta.url + '/../../..');
    const bytes = await readFile(url);
    res = await WebAssembly.instantiate(bytes, importObject);
  } else {
    throw new Error('Invalid argument: `from`');
  }
  return res.instance.exports as Exports;
}
